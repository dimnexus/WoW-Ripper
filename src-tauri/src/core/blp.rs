use serde_json::{json, Value};
use std::{collections::BTreeMap, io::{Cursor, Read}};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlpError {
    #[error("BLP header is truncated")]
    Truncated,
    #[error("unsupported BLP magic")]
    Magic,
}

pub fn inspect(bytes: &[u8]) -> Result<BTreeMap<String, Value>, BlpError> {
    if bytes.len() < 148 { return Err(BlpError::Truncated); }
    if &bytes[0..4] != b"BLP2" { return Err(BlpError::Magic); }

    let mut cur = Cursor::new(&bytes[4..]);
    let content_type = read_u32_le(&mut cur)?;
    let encoding = read_u8(&mut cur)?;
    let alpha_depth = read_u8(&mut cur)?;
    let alpha_encoding = read_u8(&mut cur)?;
    let has_mips = read_u8(&mut cur)?;
    let width = read_u32_le(&mut cur)?;
    let height = read_u32_le(&mut cur)?;

    let mut offsets = [0u32; 16];
    let mut lengths = [0u32; 16];
    for value in &mut offsets { *value = read_u32_le(&mut cur)?; }
    for value in &mut lengths { *value = read_u32_le(&mut cur)?; }

    let mip_count = offsets.iter().zip(lengths.iter()).filter(|(o,l)| **o > 0 && **l > 0).count();
    let compression = match encoding {
        1 => "Paletted",
        2 => match (alpha_depth, alpha_encoding) {
            (0 | 1, 0) => "DXT1",
            (4 | 8, 1) => "DXT3",
            (8, 7) => "DXT5",
            _ => "DXT/BCn",
        },
        3 => "Raw BGRA",
        _ => "Unknown",
    };

    let mut out = BTreeMap::new();
    out.insert("magic".into(), json!("BLP2"));
    out.insert("content_type".into(), json!(content_type));
    out.insert("encoding".into(), json!(encoding));
    out.insert("compression".into(), json!(compression));
    out.insert("alpha_depth".into(), json!(alpha_depth));
    out.insert("alpha_encoding".into(), json!(alpha_encoding));
    out.insert("has_mipmaps".into(), json!(has_mips != 0));
    out.insert("width".into(), json!(width));
    out.insert("height".into(), json!(height));
    out.insert("mipmap_count".into(), json!(mip_count));
    out.insert("mip0_offset".into(), json!(offsets[0]));
    out.insert("mip0_size".into(), json!(lengths[0]));
    Ok(out)
}

fn read_u8<R: Read>(reader: &mut R) -> Result<u8, BlpError> {
    let mut buf = [0u8;1]; reader.read_exact(&mut buf).map_err(|_| BlpError::Truncated)?; Ok(buf[0])
}
fn read_u32_le<R: Read>(reader: &mut R) -> Result<u32, BlpError> {
    let mut buf = [0u8;4]; reader.read_exact(&mut buf).map_err(|_| BlpError::Truncated)?; Ok(u32::from_le_bytes(buf))
}
