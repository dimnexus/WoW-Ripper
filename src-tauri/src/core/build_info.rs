use super::types::WowBuildInfo;
use std::{collections::BTreeMap, fs, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildInfoError {
    #[error(".build.info was not found under {0}")]
    Missing(String),
    #[error("failed to read .build.info: {0}")]
    Io(#[from] std::io::Error),
    #[error(".build.info is empty or malformed")]
    Malformed,
}

pub fn inspect_install(root: &Path) -> Result<WowBuildInfo, BuildInfoError> {
    let file = root.join(".build.info");
    if !file.is_file() {
        return Err(BuildInfoError::Missing(root.display().to_string()));
    }

    let text = fs::read_to_string(file)?;
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header = lines.next().ok_or(BuildInfoError::Malformed)?;
    let columns: Vec<String> = header.split('|').map(clean_column).collect();
    if columns.is_empty() {
        return Err(BuildInfoError::Malformed);
    }

    let mut rows = Vec::new();
    for line in lines {
        let values: Vec<&str> = line.split('|').collect();
        let mut row = BTreeMap::new();
        for (index, column) in columns.iter().enumerate() {
            row.insert(column.clone(), values.get(index).copied().unwrap_or("").trim().to_string());
        }
        rows.push(row);
    }

    let active_rows: Vec<&BTreeMap<String, String>> = rows
        .iter()
        .filter(|row| row.get("Active").map(|v| v == "1").unwrap_or(true))
        .collect();
    let preferred = active_rows.first().copied().or_else(|| rows.first()).ok_or(BuildInfoError::Malformed)?;

    let product = first_value(preferred, &["Product", "Branch", "Tags"]).unwrap_or("World of Warcraft").to_string();
    let version = first_value(preferred, &["Version", "VersionsName", "Build Name", "BuildId"])
        .unwrap_or("Unknown build")
        .to_string();

    Ok(WowBuildInfo {
        path: root.display().to_string(),
        product,
        version,
        build_key: first_owned(preferred, &["Build Key", "BuildKey"]),
        cdn_key: first_owned(preferred, &["CDN Key", "CDNKey"]),
        branch: first_owned(preferred, &["Branch"]),
        active_rows: active_rows.len(),
        rows,
    })
}

fn clean_column(raw: &str) -> String {
    raw.split('!').next().unwrap_or(raw).trim().to_string()
}

fn first_value<'a>(row: &'a BTreeMap<String, String>, names: &[&str]) -> Option<&'a str> {
    names.iter().find_map(|name| row.get(*name)).map(String::as_str).filter(|v| !v.is_empty())
}

fn first_owned(row: &BTreeMap<String, String>, names: &[&str]) -> Option<String> {
    first_value(row, names).map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::clean_column;

    #[test]
    fn strips_build_info_type_suffix() {
        assert_eq!(clean_column("Build Key!HEX:16"), "Build Key");
        assert_eq!(clean_column("Version!STRING:0"), "Version");
    }
}
