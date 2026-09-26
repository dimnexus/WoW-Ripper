use super::types::WowBuildInfo;
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildInfoError {
    #[error(".build.info was not found in {0} or its parent folder")]
    Missing(String),
    #[error("failed to read .build.info: {0}")]
    Io(#[from] std::io::Error),
    #[error(".build.info is empty or malformed")]
    Malformed,
}

pub fn inspect_install(selected: &Path) -> Result<WowBuildInfo, BuildInfoError> {
    let root = find_build_root(selected)
        .ok_or_else(|| BuildInfoError::Missing(selected.display().to_string()))?;
    let file = root.join(".build.info");

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
            row.insert(
                column.clone(),
                values.get(index).copied().unwrap_or("").trim().to_string(),
            );
        }
        rows.push(row);
    }

    let active_rows: Vec<&BTreeMap<String, String>> = rows
        .iter()
        .filter(|row| row.get("Active").map(|v| v == "1").unwrap_or(true))
        .collect();

    let folder_hint = selected
        .file_name()
        .and_then(|name| name.to_str())
        .map(normalize_token)
        .filter(|value| !value.is_empty());

    let preferred = folder_hint
        .as_deref()
        .and_then(|hint| best_matching_row(&active_rows, hint))
        .or_else(|| active_rows.first().copied())
        .or_else(|| rows.first())
        .ok_or(BuildInfoError::Malformed)?;

    let product = first_value(preferred, &["Product", "Branch", "Tags"])
        .unwrap_or("World of Warcraft")
        .to_string();
    let version = first_value(
        preferred,
        &["Version", "VersionsName", "Build Name", "BuildId"],
    )
    .unwrap_or("Unknown build")
    .to_string();

    Ok(WowBuildInfo {
        path: selected.display().to_string(),
        product,
        version,
        build_key: first_owned(preferred, &["Build Key", "BuildKey"]),
        cdn_key: first_owned(preferred, &["CDN Key", "CDNKey"]),
        branch: first_owned(preferred, &["Branch"]),
        active_rows: active_rows.len(),
        rows,
    })
}

fn find_build_root(selected: &Path) -> Option<PathBuf> {
    if selected.join(".build.info").is_file() {
        return Some(selected.to_path_buf());
    }

    selected
        .parent()
        .filter(|parent| parent.join(".build.info").is_file())
        .map(Path::to_path_buf)
}

fn best_matching_row<'a>(
    rows: &'a [&'a BTreeMap<String, String>],
    hint: &str,
) -> Option<&'a BTreeMap<String, String>> {
    rows.iter()
        .copied()
        .map(|row| (row, row_match_score(row, hint)))
        .filter(|(_, score)| *score > 0)
        .max_by_key(|(_, score)| *score)
        .map(|(row, _)| row)
}

fn row_match_score(row: &BTreeMap<String, String>, hint: &str) -> usize {
    row.values()
        .map(|value| normalize_token(value))
        .map(|value| {
            if value == hint {
                100
            } else if value.ends_with(hint) {
                80
            } else if value.contains(hint) {
                40
            } else {
                0
            }
        })
        .max()
        .unwrap_or(0)
}

fn normalize_token(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn clean_column(raw: &str) -> String {
    raw.split('!').next().unwrap_or(raw).trim().to_string()
}

fn first_value<'a>(row: &'a BTreeMap<String, String>, names: &[&str]) -> Option<&'a str> {
    names.iter()
        .find_map(|name| row.get(*name))
        .map(String::as_str)
        .filter(|v| !v.is_empty())
}

fn first_owned(row: &BTreeMap<String, String>, names: &[&str]) -> Option<String> {
    first_value(row, names).map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::{clean_column, normalize_token, row_match_score};
    use std::collections::BTreeMap;

    #[test]
    fn strips_build_info_type_suffix() {
        assert_eq!(clean_column("Build Key!HEX:16"), "Build Key");
        assert_eq!(clean_column("Version!STRING:0"), "Version");
    }

    #[test]
    fn normalizes_product_folder_names() {
        assert_eq!(normalize_token("_classic_beta_"), "classicbeta");
        assert_eq!(normalize_token("wow_classic_beta"), "wowclassicbeta");
    }

    #[test]
    fn prefers_specific_classic_branch() {
        let mut row = BTreeMap::new();
        row.insert("Product".to_string(), "wow_classic_beta".to_string());
        assert_eq!(row_match_score(&row, "classicbeta"), 80);
        assert_eq!(row_match_score(&row, "classic"), 40);
    }
}
