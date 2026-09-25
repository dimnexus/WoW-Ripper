use super::types::{ExtensionCount, ListfileMatch, ListfileSummary};
use std::{collections::HashMap, fs, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListfileError {
    #[error("failed to read ListFile: {0}")]
    Io(#[from] std::io::Error),
    #[error("ListFile contained no FileDataID/path rows")]
    Empty,
}

#[derive(Debug, Clone, Default)]
pub struct ListfileIndex {
    entries: Vec<ListfileMatch>,
}

impl ListfileIndex {
    pub fn load(path: &Path) -> Result<(Self, ListfileSummary), ListfileError> {
        let text = fs::read_to_string(path)?;
        let mut entries = Vec::new();
        let mut extensions = HashMap::<String, usize>::new();

        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            let mut parts = line.splitn(2, ';');
            let id = parts.next().unwrap_or("").trim().trim_start_matches('﻿');
            let path = parts.next().unwrap_or("").trim();
            let Ok(file_data_id) = id.parse::<u32>() else { continue; };
            if path.is_empty() { continue; }

            let extension = extension_of(path);
            *extensions.entry(extension).or_insert(0) += 1;
            entries.push(ListfileMatch { file_data_id, path: path.to_string() });
        }

        if entries.is_empty() { return Err(ListfileError::Empty); }
        entries.sort_by_key(|entry| entry.file_data_id);

        let mut extension_rows: Vec<ExtensionCount> = extensions
            .into_iter()
            .map(|(extension, count)| ExtensionCount { extension, count })
            .collect();
        extension_rows.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.extension.cmp(&b.extension)));

        let summary = ListfileSummary { entries: entries.len(), extensions: extension_rows };
        Ok((Self { entries }, summary))
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<ListfileMatch> {
        let needle = query.trim().to_ascii_lowercase();
        if needle.is_empty() { return Vec::new(); }

        let exact_id = needle.parse::<u32>().ok();
        let mut out = Vec::new();
        for entry in &self.entries {
            let matches = exact_id.map(|id| id == entry.file_data_id).unwrap_or(false)
                || entry.path.to_ascii_lowercase().contains(&needle)
                || entry.file_data_id.to_string().starts_with(&needle);
            if matches {
                out.push(entry.clone());
                if out.len() >= limit { break; }
            }
        }
        out
    }
}

fn extension_of(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    normalized.rsplit('.').next().filter(|part| !part.contains('/'))
        .map(|part| part.to_ascii_lowercase()).unwrap_or_else(|| "(none)".to_string())
}

#[cfg(test)]
mod tests {
    use super::extension_of;
    #[test]
    fn extracts_extension() {
        assert_eq!(extension_of("interface/icons/test.blp"), "blp");
        assert_eq!(extension_of("path/noext"), "(none)");
    }
}
