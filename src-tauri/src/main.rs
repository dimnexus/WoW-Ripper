mod core;

use core::{
    blte,
    build_info,
    casc::{self, CascSession},
    inspector,
    installs,
    listfile::ListfileIndex,
    types::{
        AssetInspection, CascCatalogInfo, CascDirectoryListing, CascExtractResult, DecodeResult,
        InstallCandidate, ListfileMatch, ListfileSummary, WowBuildInfo,
    },
};
use std::{fs, path::PathBuf, sync::Mutex};

#[derive(Default)]
struct AppState {
    listfile: Mutex<Option<ListfileIndex>>,
    listfile_path: Mutex<Option<PathBuf>>,
    casc: Mutex<Option<CascSession>>,
}

#[tauri::command]
fn scan_installs() -> Vec<InstallCandidate> {
    installs::scan()
}

#[tauri::command]
fn choose_folder(initial_path: Option<String>) -> Option<String> {
    let mut dialog = rfd::FileDialog::new();

    if let Some(initial) = initial_path.as_deref().filter(|value| !value.trim().is_empty()) {
        let path = user_path(initial);
        if path.is_dir() {
            dialog = dialog.set_directory(path);
        }
    }

    dialog.pick_folder().map(|path| path.display().to_string())
}

#[tauri::command]
fn inspect_wow_install(path: String) -> Result<WowBuildInfo, String> {
    build_info::inspect_install(&user_path(&path)).map_err(|err| err.to_string())
}

#[tauri::command]
fn open_casc_catalog(path: String, state: tauri::State<'_, AppState>) -> Result<CascCatalogInfo, String> {
    let selected = user_path(&path);
    let custom_listfile = state
        .listfile_path
        .lock()
        .map_err(|_| "ListFile path state lock poisoned".to_string())?
        .clone();

    let (session, info) = CascSession::open(&selected, custom_listfile.as_deref())
        .map_err(|err| err.to_string())?;

    let (runtime_index, _) = ListfileIndex::from_entries(session.files().to_vec());
    *state.listfile.lock().map_err(|_| "ListFile state lock poisoned".to_string())? = Some(runtime_index);
    *state.casc.lock().map_err(|_| "CASC state lock poisoned".to_string())? = Some(session);

    Ok(info)
}

#[tauri::command]
fn browse_casc_directory(path: String, state: tauri::State<'_, AppState>) -> Result<CascDirectoryListing, String> {
    let guard = state.casc.lock().map_err(|_| "CASC state lock poisoned".to_string())?;
    let session = guard.as_ref().ok_or_else(|| "Choose a WoW build before browsing CASC".to_string())?;
    Ok(session.list_directory(&path))
}

#[tauri::command]
fn inspect_casc_file(file_data_id: u32, state: tauri::State<'_, AppState>) -> Result<AssetInspection, String> {
    let guard = state.casc.lock().map_err(|_| "CASC state lock poisoned".to_string())?;
    let session = guard.as_ref().ok_or_else(|| "Choose a WoW build before inspecting CASC files".to_string())?;
    session.inspect_file(file_data_id).map_err(|err| err.to_string())
}

#[tauri::command]
fn extract_casc_file(
    file_data_id: u32,
    output_root: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<CascExtractResult, String> {
    let output = output_root
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(user_path);

    let guard = state.casc.lock().map_err(|_| "CASC state lock poisoned".to_string())?;
    let session = guard.as_ref().ok_or_else(|| "Choose a WoW build before extracting CASC files".to_string())?;
    session.extract_file(file_data_id, output.as_deref()).map_err(|err| err.to_string())
}

#[tauri::command]
fn default_extract_root() -> String {
    casc::default_extract_root().display().to_string()
}

#[tauri::command]
fn import_listfile(path: String, state: tauri::State<'_, AppState>) -> Result<ListfileSummary, String> {
    let cleaned = user_path(&path);
    let (index, summary) = ListfileIndex::load(&cleaned).map_err(|err| err.to_string())?;
    *state.listfile.lock().map_err(|_| "ListFile state lock poisoned".to_string())? = Some(index);
    *state.listfile_path.lock().map_err(|_| "ListFile path state lock poisoned".to_string())? = Some(cleaned);
    Ok(summary)
}

#[tauri::command]
fn search_listfile(query: String, limit: Option<usize>, state: tauri::State<'_, AppState>) -> Result<Vec<ListfileMatch>, String> {
    let guard = state.listfile.lock().map_err(|_| "ListFile state lock poisoned".to_string())?;
    let index = guard.as_ref().ok_or_else(|| "Load a WoW build or import a ListFile first".to_string())?;
    Ok(index.search(&query, limit.unwrap_or(250).clamp(1, 2000)))
}

#[tauri::command]
fn inspect_asset(path: String) -> Result<AssetInspection, String> {
    inspector::inspect(&user_path(&path)).map_err(|err| err.to_string())
}

#[tauri::command]
fn decode_blte_file(input_path: String, output_path: String) -> Result<DecodeResult, String> {
    let input = user_path(&input_path);
    let output = user_path(&output_path);
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

fn user_path(value: &str) -> PathBuf {
    let trimmed = value.trim();

    let unquoted = if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    PathBuf::from(unquoted)
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            scan_installs,
            choose_folder,
            inspect_wow_install,
            open_casc_catalog,
            browse_casc_directory,
            inspect_casc_file,
            extract_casc_file,
            default_extract_root,
            import_listfile,
            search_listfile,
            inspect_asset,
            decode_blte_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running WoW Ripper");
}
