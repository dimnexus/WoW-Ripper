use serde_json::{json, Value};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("table header is truncated")]
    Truncated,
    #[error("not a recognized DBC/DB2 family signature")]
    Magic,
}

pub fn inspect(bytes: &[u8]) -> Result<BTreeMap<String, Value>, DbError> {
    if bytes.len() < 20 { return Err(DbError::Truncated); }
    let magic = std::str::from_utf8(&bytes[0..4]).unwrap_or("????");
    let recognized = matches!(magic, "WDBC" | "WDB2" | "WDB5" | "WDB6" | "WDC1" | "WDC2" | "WDC3" | "WDC4" | "WDC5" | "WDC6");
    if !recognized { return Err(DbError::Magic); }

    let record_count = le_u32(bytes, 4)?;
    let field_count = le_u32(bytes, 8)?;
    let record_size = le_u32(bytes, 12)?;
    let string_table_size = le_u32(bytes, 16)?;

    let mut out = BTreeMap::new();
    out.insert("signature".into(), json!(magic));
    out.insert("record_count".into(), json!(record_count));
    out.insert("field_count".into(), json!(field_count));
    out.insert("record_size".into(), json!(record_size));
    out.insert("string_table_size".into(), json!(string_table_size));

    if matches!(magic, "WDB2" | "WDB5" | "WDB6") && bytes.len() >= 48 {
        out.insert("table_hash".into(), json!(le_u32(bytes, 20)?));
        out.insert("build".into(), json!(le_u32(bytes, 24)?));
        out.insert("min_id".into(), json!(le_u32(bytes, 32)?));
        out.insert("max_id".into(), json!(le_u32(bytes, 36)?));
    }
    Ok(out)
}

fn le_u32(bytes: &[u8], offset: usize) -> Result<u32, DbError> {
    let slice = bytes.get(offset..offset + 4).ok_or(DbError::Truncated)?;
    Ok(u32::from_le_bytes(slice.try_into().unwrap()))
}
