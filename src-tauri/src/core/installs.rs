use super::types::InstallCandidate;
use std::{collections::BTreeSet, env, path::{Path, PathBuf}};

pub fn scan() -> Vec<InstallCandidate> {
    let mut roots = BTreeSet::<PathBuf>::new();

    for key in ["PROGRAMFILES(X86)", "PROGRAMFILES"] {
        if let Ok(value) = env::var(key) {
            roots.insert(PathBuf::from(value).join("World of Warcraft"));
        }
    }

    for drive in ['C', 'D', 'E', 'F'] {
        roots.insert(PathBuf::from(format!(r"{}:Program Files (x86)World of Warcraft", drive)));
        roots.insert(PathBuf::from(format!(r"{}:Program FilesWorld of Warcraft", drive)));
        roots.insert(PathBuf::from(format!(r"{}:World of Warcraft", drive)));
    }

    if let Ok(home) = env::var("HOME") {
        roots.insert(PathBuf::from(&home).join(".wine/drive_c/Program Files (x86)/World of Warcraft"));
        roots.insert(PathBuf::from(&home).join("Games/World of Warcraft"));
    }
    roots.insert(PathBuf::from("/mnt/c/Program Files (x86)/World of Warcraft"));

    roots
        .into_iter()
        .filter(|path| path.is_dir())
        .map(|path| candidate(path.as_path()))
        .collect()
}

fn candidate(path: &Path) -> InstallCandidate {
    InstallCandidate {
        path: path.display().to_string(),
        label: path.file_name().and_then(|n| n.to_str()).unwrap_or("World of Warcraft").to_string(),
        has_build_info: path.join(".build.info").is_file(),
    }
}
