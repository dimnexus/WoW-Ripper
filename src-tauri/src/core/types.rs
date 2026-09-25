use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct InstallCandidate {
    pub path: String,
    pub label: String,
    pub has_build_info: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WowBuildInfo {
    pub path: String,
    pub product: String,
    pub version: String,
    pub build_key: Option<String>,
    pub cdn_key: Option<String>,
    pub branch: Option<String>,
    pub active_rows: usize,
    pub rows: Vec<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListfileSummary {
    pub entries: usize,
    pub extensions: Vec<ExtensionCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtensionCount {
    pub extension: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListfileMatch {
    pub file_data_id: u32,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetInspection {
    pub path: String,
    pub size: u64,
    pub format: String,
    pub details: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecodeResult {
    pub input_path: String,
    pub output_path: String,
    pub encoded_bytes: u64,
    pub decoded_bytes: u64,
    pub chunks: usize,
}
