use super::{
    build_info,
    inspector,
    types::{
        AssetInspection, CascCatalogInfo, CascDirectoryListing, CascExtractResult,
        CascVirtualEntry, ListfileMatch,
    },
};
use casc_lib::{
    extract::{list_files, CascStorage, OpenConfig},
    root::flags::LocaleFlags,
};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Component, Path, PathBuf},
};
use thiserror::Error;

const ALL_LOCALES: u32 = 0xFFFF_FFFF;

#[derive(Debug, Error)]
pub enum CascSessionError {
    #[error("could not locate the shared WoW install root for {0}")]
    InstallRoot(String),
    #[error("CASC open failed: {0}")]
    Open(String),
    #[error("FileDataID {0} is not in the loaded catalog")]
    MissingFile(u32),
    #[error("failed to write extracted file: {0}")]
    Write(#[from] std::io::Error),
}

pub struct CascSession {
    storage: CascStorage,
    files: Vec<ListfileMatch>,
    selected_path: PathBuf,
    install_root: PathBuf,
    product: String,
}

impl CascSession {
    pub fn open(selected: &Path, custom_listfile: Option<&Path>) -> Result<(Self, CascCatalogInfo), CascSessionError> {
        let build = build_info::inspect_install(selected)
            .map_err(|err| CascSessionError::Open(err.to_string()))?;
        let install_root = find_install_root(selected)
            .ok_or_else(|| CascSessionError::InstallRoot(selected.display().to_string()))?;

        let config = OpenConfig {
            install_dir: install_root.clone(),
            product: Some(build.product.clone()),
            keyfile: None,
            listfile: custom_listfile.map(Path::to_path_buf),
            output_dir: Some(cache_dir()),
        };

        let storage = CascStorage::open(&config)
            .map_err(|err| CascSessionError::Open(err.to_string()))?;
        let storage_info = storage.info();

        let mut files: Vec<ListfileMatch> = list_files(&storage, ALL_LOCALES, None)
            .into_iter()
            .filter(|(_, path)| !path.eq_ignore_ascii_case("unknown"))
            .map(|(file_data_id, path)| ListfileMatch {
                file_data_id,
                path: normalize_virtual_path(&path),
            })
            .collect();

        files.sort_by(|a, b| {
            a.path
                .to_ascii_lowercase()
                .cmp(&b.path.to_ascii_lowercase())
                .then_with(|| a.file_data_id.cmp(&b.file_data_id))
        });
        files.dedup_by(|a, b| a.file_data_id == b.file_data_id);

        let info = CascCatalogInfo {
            selected_path: selected.display().to_string(),
            install_root: install_root.display().to_string(),
            product: storage_info.product,
            version: storage_info.version,
            build_name: storage_info.build_name,
            root_format: storage_info.root_format,
            files: files.len(),
            encoding_entries: storage_info.encoding_entries,
            root_entries: storage_info.root_entries,
            index_entries: storage_info.index_entries,
            listfile_entries: storage_info.listfile_entries,
            default_extract_root: default_extract_root().display().to_string(),
        };

        Ok((
            Self {
                storage,
                files,
                selected_path: selected.to_path_buf(),
                install_root,
                product: build.product,
            },
            info,
        ))
    }

    pub fn files(&self) -> &[ListfileMatch] {
        &self.files
    }

    pub fn list_directory(&self, requested: &str) -> CascDirectoryListing {
        let current = normalize_directory(requested);
        let prefix = if current.is_empty() {
            String::new()
        } else {
            format!("{current}/")
        };
        let prefix_lower = prefix.to_ascii_lowercase();

        let mut folders = BTreeMap::<String, CascVirtualEntry>::new();
        let mut files = Vec::<CascVirtualEntry>::new();

        for file in &self.files {
            let normalized = normalize_virtual_path(&file.path);
            let lower = normalized.to_ascii_lowercase();
            if !lower.starts_with(&prefix_lower) {
                continue;
            }

            let rest = &normalized[prefix.len()..];
            if rest.is_empty() {
                continue;
            }

            if let Some(split) = rest.find('/') {
                let name = &rest[..split];
                let key = name.to_ascii_lowercase();
                let path = if current.is_empty() {
                    name.to_string()
                } else {
                    format!("{current}/{name}")
                };
                folders.entry(key).or_insert_with(|| CascVirtualEntry {
                    name: name.to_string(),
                    path,
                    kind: "folder".into(),
                    file_data_id: None,
                    extension: None,
                });
            } else {
                files.push(CascVirtualEntry {
                    name: rest.to_string(),
                    path: normalized,
                    kind: "file".into(),
                    file_data_id: Some(file.file_data_id),
                    extension: extension_of(rest),
                });
            }
        }

        files.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()));

        CascDirectoryListing {
            path: current,
            folders: folders.into_values().collect(),
            files,
        }
    }

    pub fn inspect_file(&self, file_data_id: u32) -> Result<AssetInspection, CascSessionError> {
        let path = self
            .files
            .iter()
            .find(|entry| entry.file_data_id == file_data_id)
            .map(|entry| entry.path.clone())
            .ok_or(CascSessionError::MissingFile(file_data_id))?;

        let bytes = self
            .storage
            .read_by_fdid(file_data_id, LocaleFlags(ALL_LOCALES))
            .map_err(|err| CascSessionError::Open(err.to_string()))?;

        Ok(inspector::inspect_bytes(&path, &bytes))
    }

    pub fn extract_file(
        &self,
        file_data_id: u32,
        output_root: Option<&Path>,
    ) -> Result<CascExtractResult, CascSessionError> {
        let virtual_path = self
            .files
            .iter()
            .find(|entry| entry.file_data_id == file_data_id)
            .map(|entry| entry.path.clone())
            .ok_or(CascSessionError::MissingFile(file_data_id))?;

        let bytes = self
            .storage
            .read_by_fdid(file_data_id, LocaleFlags(ALL_LOCALES))
            .map_err(|err| CascSessionError::Open(err.to_string()))?;

        let root = output_root
            .map(Path::to_path_buf)
            .unwrap_or_else(default_extract_root);
        let relative = safe_virtual_path(&virtual_path)
            .unwrap_or_else(|| PathBuf::from("unknown").join(format!("{file_data_id}.dat")));
        let output = root.join(relative);

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output, &bytes)?;

        Ok(CascExtractResult {
            file_data_id,
            virtual_path,
            output_path: output.display().to_string(),
            bytes: bytes.len() as u64,
        })
    }

    #[allow(dead_code)]
    pub fn selected_path(&self) -> &Path {
        &self.selected_path
    }

    #[allow(dead_code)]
    pub fn install_root(&self) -> &Path {
        &self.install_root
    }

    #[allow(dead_code)]
    pub fn product(&self) -> &str {
        &self.product
    }
}

pub fn default_extract_root() -> PathBuf {
    if let Ok(profile) = env::var("USERPROFILE") {
        return PathBuf::from(profile)
            .join("Documents")
            .join("WoW Ripper")
            .join("Extracted");
    }
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join("WoW Ripper").join("Extracted");
    }
    PathBuf::from("WoW-Ripper-Extracted")
}

fn cache_dir() -> PathBuf {
    if let Ok(local) = env::var("LOCALAPPDATA") {
        return PathBuf::from(local).join("WoW Ripper").join("cache");
    }
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join(".cache").join("wow-ripper");
    }
    env::temp_dir().join("wow-ripper")
}

fn find_install_root(selected: &Path) -> Option<PathBuf> {
    if selected.join(".build.info").is_file() && selected.join("Data").is_dir() {
        return Some(selected.to_path_buf());
    }

    selected.parent().and_then(|parent| {
        if parent.join(".build.info").is_file() && parent.join("Data").is_dir() {
            Some(parent.to_path_buf())
        } else {
            None
        }
    })
}

fn normalize_virtual_path(path: &str) -> String {
    path.replace('\\', "/")
        .trim_matches('/')
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("/")
}

fn normalize_directory(path: &str) -> String {
    normalize_virtual_path(path)
}

fn extension_of(name: &str) -> Option<String> {
    name.rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
}

fn safe_virtual_path(path: &str) -> Option<PathBuf> {
    let normalized = normalize_virtual_path(path);
    let mut out = PathBuf::new();

    for component in Path::new(&normalized).components() {
        match component {
            Component::Normal(value) => out.push(value),
            _ => return None,
        }
    }

    if out.as_os_str().is_empty() { None } else { Some(out) }
}

#[cfg(test)]
mod tests {
    use super::{normalize_virtual_path, safe_virtual_path};

    #[test]
    fn normalizes_casc_paths() {
        assert_eq!(normalize_virtual_path(r"Interface\Icons\INV_Test.blp"), "Interface/Icons/INV_Test.blp");
    }

    #[test]
    fn rejects_parent_traversal() {
        assert!(safe_virtual_path("../../secret.txt").is_none());
    }
}
