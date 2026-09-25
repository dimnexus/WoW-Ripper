mod core;

use core::{
    blte,
    build_info,
    inspector,
    installs,
    listfile::ListfileIndex,
    types::{AssetInspection, DecodeResult, InstallCandidate, ListfileMatch, ListfileSummary, WowBuildInfo},
};
use std::{fs, path::PathBuf, sync::Mutex};

#[derive(Default)]
struct AppState {
    listfile: Mutex<Option<ListfileIndex>>,
}

#[tauri::command]
fn scan_installs() -> Vec<InstallCandidate> {
    installs::scan()
}

#[tauri::command]
fn inspect_wow_install(path: String) -> Result<WowBuildInfo, String> {
    build_info::inspect_install(&PathBuf::from(path)).map_err(|err| err.to_string())
}

#[tauri::command]
fn import_listfile(path: String, state: tauri::State<'_, AppState>) -> Result<ListfileSummary, String> {
    let (index, summary) = ListfileIndex::load(&PathBuf::from(path)).map_err(|err| err.to_string())?;
    *state.listfile.lock().map_err(|_| "ListFile state lock poisoned".to_string())? = Some(index);
    Ok(summary)
}

#[tauri::command]
fn search_listfile(query: String, limit: Option<usize>, state: tauri::State<'_, AppState>) -> Result<Vec<ListfileMatch>, String> {
    let guard = state.listfile.lock().map_err(|_| "ListFile state lock poisoned".to_string())?;
    let index = guard.as_ref().ok_or_else(|| "Import a ListFile first".to_string())?;
    Ok(index.search(&query, limit.unwrap_or(250).clamp(1, 2000)))
}

#[tauri::command]
fn inspect_asset(path: String) -> Result<AssetInspection, String> {
    inspector::inspect(&PathBuf::from(path)).map_err(|err| err.to_string())
}

#[tauri::command]
fn decode_blte_file(input_path: String, output_path: String) -> Result<DecodeResult, String> {
    let input = PathBuf::from(&input_path);
    let output = PathBuf::from(&output_path);
    let encoded = fs::read(&input).map_err(|err| err.to_string())?;
    let decoded = blte::decode(&encoded).map_err(|err| err.to_string())?;
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() { fs::create_dir_all(parent).map_err(|err| err.to_string())?; }
    }
    fs::write(&output, &decoded.bytes).map_err(|err| err.to_string())?;
    Ok(DecodeResult {
        input_path,
        output_path,
        encoded_bytes: encoded.len() as u64,
        decoded_bytes: decoded.bytes.len() as u64,
        chunks: decoded.chunks,
    })
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            scan_installs,
            inspect_wow_install,
            import_listfile,
            search_listfile,
            inspect_asset,
            decode_blte_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running WoW Ripper");
}
