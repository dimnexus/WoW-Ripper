use super::{blp, db2, types::AssetInspection};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InspectError {
    #[error("failed to read asset: {0}")]
    Io(#[from] std::io::Error),
}

pub fn inspect(path: &Path) -> Result<AssetInspection, InspectError> {
    let bytes = fs::read(path)?;
    Ok(inspect_bytes(&path.display().to_string(), &bytes))
}

pub fn inspect_bytes(label: &str, bytes: &[u8]) -> AssetInspection {
    let size = bytes.len() as u64;
    let mut details = BTreeMap::new();
    let format = if bytes.starts_with(b"BLP2") {
        match blp::inspect(bytes) { Ok(map) => { details = map; "BLP2" }, Err(err) => { details.insert("error".into(), json!(err.to_string())); "BLP2" } }
    } else if bytes.starts_with(b"BLTE") {
        let header_size = bytes.get(4..8).map(|s| u32::from_be_bytes(s.try_into().unwrap())).unwrap_or(0);
        details.insert("header_size".into(), json!(header_size));
        details.insert("decoder_modes".into(), json!(["N/raw", "Z/zlib"]));
        "BLTE"
    } else if is_db_signature(bytes) {
        match db2::inspect(bytes) { Ok(map) => { details = map; "DBC/DB2" }, Err(err) => { details.insert("error".into(), json!(err.to_string())); "DBC/DB2" } }
    } else if bytes.starts_with(b"MD20") || bytes.starts_with(b"MD21") {
        details.insert("magic".into(), json!(String::from_utf8_lossy(&bytes[0..4]).to_string()));
        "M2 model"
    } else if bytes.starts_with(b"REVM") || bytes.starts_with(b"MVER") {
        details.insert("magic".into(), json!(String::from_utf8_lossy(&bytes[0..4]).to_string()));
        "Chunked WoW asset"
    } else if bytes.starts_with(b"OggS") {
        "Ogg Vorbis"
    } else if bytes.starts_with(b"ID3") || bytes.starts_with(&[0xFF, 0xFB]) || bytes.starts_with(&[0xFF, 0xF3]) {
        "MP3"
    } else {
        if bytes.len() >= 4 { details.insert("magic_hex".into(), json!(format!("{:02X} {:02X} {:02X} {:02X}", bytes[0], bytes[1], bytes[2], bytes[3]))); }
        "Unknown"
    };

    AssetInspection { path: label.to_string(), size, format: format.to_string(), details }
}

fn is_db_signature(bytes: &[u8]) -> bool {
    if bytes.len() < 4 { return false; }
    let magic = &bytes[0..4];
    [b"WDBC".as_slice(), b"WDB2".as_slice(), b"WDB5".as_slice(), b"WDB6".as_slice(), b"WDC1".as_slice(), b"WDC2".as_slice(), b"WDC3".as_slice(), b"WDC4".as_slice(), b"WDC5".as_slice(), b"WDC6".as_slice()]
        .iter()
        .any(|candidate| magic == *candidate)
}
