const $ = (id) => document.getElementById(id);
const invoke = window.__TAURI__?.core?.invoke;
const SAVED_WOW_PATH_KEY = 'wowRipper.wowInstallPath';
const SAVED_EXTRACT_ROOT_KEY = 'wowRipper.extractRoot';

const state = {
  view: 'home',
  build: null,
  cascInfo: null,
  cascReady: false,
  currentFolder: '',
  selectedCascFile: null,
  listfileCount: 0,
  queue: [],
  luaSelected: new Map(),
};

function log(message, payload) {
  const box = $('eventLog');
  if (!box) return;
  const stamp = new Date().toLocaleTimeString();
  const text = payload ? `${message}\n${JSON.stringify(payload, null, 2)}` : message;
  box.textContent = `[${stamp}] ${text}\n\n${box.textContent}`.slice(0, 16000);
}

function showView(name) {
  state.view = name;
  document.querySelectorAll('.view').forEach((v) => v.classList.remove('active'));
  document.querySelectorAll('.nav-item').forEach((v) => v.classList.remove('active'));
  $(`view-${name}`)?.classList.add('active');
  document.querySelector(`.nav-item[data-view="${name}"]`)?.classList.add('active');
  const names = { home:'Dashboard', files:'CASC Browser', textures:'Texture Studio', models:'Model Studio', world:'World Studio', audio:'Audio Browser', data:'DB2 Studio', addon:'Addon Lab', diagnostics:'Diagnostics' };
  $('viewTitle').textContent = names[name] || name;
}

document.querySelectorAll('[data-view]').forEach((b) => b.addEventListener('click', () => showView(b.dataset.view)));
document.querySelectorAll('[data-jump]').forEach((b) => b.addEventListener('click', () => showView(b.dataset.jump)));

function ensureDesktop() {
  if (invoke) return true;
  log('Desktop bridge unavailable. Run through the Tauri application to use local-file commands.');
  return false;
}

async function call(command, args = {}) {
  if (!ensureDesktop()) throw new Error('Tauri desktop bridge unavailable');
  try {
    return await invoke(command, args);
  } catch (err) {
    log(`${command} failed`, String(err));
    throw err;
  }
}

function updateBuildUI(build) {
  state.build = build;
  const pill = $('buildPill');
  if (!build) {
    pill.innerHTML = '<span class="status-dot idle"></span><span>No build loaded</span>';
    $('metricBuild').textContent = 'None';
    $('metricProduct').textContent = 'Choose your WoW installation folder';
    return;
  }
  pill.innerHTML = `<span class="status-dot good"></span><span>${escapeHtml(build.version || build.product || 'WoW build')}</span>`;
  $('metricBuild').textContent = build.version || 'Detected';
  $('metricProduct').textContent = build.product || build.path || 'World of Warcraft';
}

function updateQueueUI() {
  $('metricQueue').textContent = state.queue.length;
  $('queueCount').textContent = `${state.queue.length} job${state.queue.length === 1 ? '' : 's'}`;
  $('queueSummary').textContent = state.queue.length ? state.queue[state.queue.length - 1].label : 'Idle';
}

function addQueue(label, status = 'completed') {
  state.queue.push({ label, status, at: Date.now() });
  updateQueueUI();
}

$('clearQueue').addEventListener('click', () => { state.queue.length = 0; updateQueueUI(); });

function setCascStatus(text, mode = '') {
  const el = $('cascStatus');
  el.className = `casc-status ${mode}`.trim();
  const dotClass = mode === 'ready' ? 'good' : mode === 'error' ? 'warn' : 'idle';
  el.innerHTML = `<span class="status-dot ${dotClass}"></span><span>${escapeHtml(text)}</span>`;
}

async function openCascCatalog(selectedPath) {
  state.cascReady = false;
  state.selectedCascFile = null;
  setCascStatus('Opening CASC storage, root, encoding table, and FileDataID catalog...');
  clearCascSelection();

  try {
    const info = await call('open_casc_catalog', { path: selectedPath });
    state.cascInfo = info;
    state.cascReady = true;
    state.currentFolder = '';
    state.listfileCount = info.files;
    $('metricListfile').textContent = Number(info.files).toLocaleString();

    if (!$('extractRoot').value.trim()) {
      $('extractRoot').value = localStorage.getItem(SAVED_EXTRACT_ROOT_KEY) || info.default_extract_root || '';
    }

    setCascStatus(`${info.product} ${info.version} · ${Number(info.files).toLocaleString()} named files · ${info.root_format} root`, 'ready');
    log('CASC catalog opened', info);
    await refreshCascDirectory();
  } catch (err) {
    state.cascInfo = null;
    state.cascReady = false;
    setCascStatus(`CASC could not be opened: ${String(err)}`, 'error');
    $('cascFolders').className = 'browser-list empty-state';
    $('cascFolders').textContent = 'CASC open failed. See Diagnostics for the exact error.';
    $('cascFiles').className = 'browser-list empty-state';
    $('cascFiles').textContent = 'No files loaded.';
  }
}

$('openBuildButton').addEventListener('click', async () => {
  const remembered = localStorage.getItem(SAVED_WOW_PATH_KEY) || '';

  try {
    const path = await call('choose_folder', { initialPath: remembered || null });
    if (!path) return;

    const info = await call('inspect_wow_install', { path });
    localStorage.setItem(SAVED_WOW_PATH_KEY, path);
    updateBuildUI(info);
    log('Loaded build from selected folder', info);
    await openCascCatalog(path);
    showView('files');
  } catch (err) { /* logged */ }
});

async function loadRememberedBuild() {
  const path = localStorage.getItem(SAVED_WOW_PATH_KEY);
  if (!path || !invoke) return;
  try {
    const info = await call('inspect_wow_install', { path });
    updateBuildUI(info);
    log('Reloaded remembered WoW folder', { path });
    await openCascCatalog(path);
  } catch (err) {
    log('Remembered WoW folder could not be loaded. Choose the folder again.', { path });
  }
}

async function refreshCascDirectory() {
  if (!state.cascReady) return;
  const listing = await call('browse_casc_directory', { path: state.currentFolder });
  state.currentFolder = listing.path || '';
  renderCascDirectory(listing);
}

function renderCascDirectory(listing) {
  $('cascBreadcrumb').textContent = listing.path ? `/${listing.path}` : '/';
  $('folderCount').textContent = listing.folders.length;
  $('fileCount').textContent = listing.files.length;

  const folders = $('cascFolders');
  folders.className = 'browser-list';
  folders.innerHTML = listing.folders.length
    ? listing.folders.map((entry) => `<div class="browser-row folder" data-folder="${escapeAttr(entry.path)}"><span class="browser-icon">DIR</span><span class="browser-name" title="${escapeAttr(entry.path)}">${escapeHtml(entry.name)}</span></div>`).join('')
    : '<div class="empty-state">No subfolders.</div>';

  folders.querySelectorAll('[data-folder]').forEach((row) => row.addEventListener('click', async () => {
    state.currentFolder = row.dataset.folder;
    clearCascSelection();
    await refreshCascDirectory();
  }));

  const files = $('cascFiles');
  files.className = 'browser-list';
  files.innerHTML = listing.files.length
    ? listing.files.map((entry) => `<div class="browser-row file" data-fdid="${entry.file_data_id}" data-path="${escapeAttr(entry.path)}" data-name="${escapeAttr(entry.name)}" data-ext="${escapeAttr(entry.extension || '')}"><span class="browser-id">${entry.file_data_id}</span><span class="browser-name" title="${escapeAttr(entry.path)}">${escapeHtml(entry.name)}</span><span class="browser-ext">${escapeHtml(entry.extension || '')}</span></div>`).join('')
    : '<div class="empty-state">No named files directly in this folder.</div>';

  files.querySelectorAll('[data-fdid]').forEach((row) => row.addEventListener('click', () => selectCascFile(row)));
}

function selectCascFile(row) {
  $('cascFiles').querySelectorAll('.browser-row').forEach((item) => item.classList.remove('selected'));
  row.classList.add('selected');

  state.selectedCascFile = {
    file_data_id: Number(row.dataset.fdid),
    path: row.dataset.path,
    name: row.dataset.name,
    extension: row.dataset.ext,
  };

  $('cascSelection').className = 'selection-card';
  $('cascSelection').innerHTML = `<div class="selection-path">${escapeHtml(state.selectedCascFile.path)}</div><div class="selection-meta"><span>FileDataID <strong>${state.selectedCascFile.file_data_id}</strong></span><span>${escapeHtml((state.selectedCascFile.extension || 'file').toUpperCase())}</span></div>`;
  $('cascInspectButton').disabled = false;
  $('cascExtractButton').disabled = false;
  $('cascInspectResult').className = 'inspection empty-state compact-inspection';
  $('cascInspectResult').textContent = 'Click Inspect to read this file directly from CASC.';
}

function clearCascSelection() {
  state.selectedCascFile = null;
  if ($('cascSelection')) {
    $('cascSelection').className = 'selection-card empty-state';
    $('cascSelection').textContent = 'Select a file to inspect or extract it.';
    $('cascInspectButton').disabled = true;
    $('cascExtractButton').disabled = true;
  }
}

$('cascHomeButton').addEventListener('click', async () => {
  state.currentFolder = '';
  clearCascSelection();
  await refreshCascDirectory().catch(() => {});
});

$('cascUpButton').addEventListener('click', async () => {
  if (!state.currentFolder) return;
  const parts = state.currentFolder.split('/');
  parts.pop();
  state.currentFolder = parts.join('/');
  clearCascSelection();
  await refreshCascDirectory().catch(() => {});
});

$('cascInspectButton').addEventListener('click', async () => {
  if (!state.selectedCascFile) return;
  $('cascInspectButton').disabled = true;
  try {
    const info = await call('inspect_casc_file', { fileDataId: state.selectedCascFile.file_data_id });
    renderInspection(info, $('cascInspectResult'));
    log('CASC file inspected', info);
  } catch (err) {
    $('cascInspectResult').className = 'inspection empty-state compact-inspection';
    $('cascInspectResult').textContent = String(err);
  } finally {
    $('cascInspectButton').disabled = false;
  }
});

$('cascExtractButton').addEventListener('click', async () => {
  if (!state.selectedCascFile) return;
  const outputRoot = $('extractRoot').value.trim();
  if (outputRoot) localStorage.setItem(SAVED_EXTRACT_ROOT_KEY, outputRoot);

  $('cascExtractButton').disabled = true;
  try {
    const result = await call('extract_casc_file', {
      fileDataId: state.selectedCascFile.file_data_id,
      outputRoot: outputRoot || null,
    });
    addQueue(`Extracted ${state.selectedCascFile.name}`);
    log('CASC file extracted', result);
    $('cascSelection').innerHTML += `<div class="selection-meta"><span>Saved: ${escapeHtml(result.output_path)}</span></div>`;
  } catch (err) {
    log('CASC extraction failed', String(err));
  } finally {
    $('cascExtractButton').disabled = false;
  }
});

$('extractRoot').addEventListener('change', () => {
  const value = $('extractRoot').value.trim();
  if (value) localStorage.setItem(SAVED_EXTRACT_ROOT_KEY, value);
});

$('chooseExtractRoot').addEventListener('click', async () => {
  const remembered = $('extractRoot').value.trim() || localStorage.getItem(SAVED_EXTRACT_ROOT_KEY) || '';
  try {
    const path = await call('choose_folder', { initialPath: remembered || null });
    if (!path) return;
    $('extractRoot').value = path;
    localStorage.setItem(SAVED_EXTRACT_ROOT_KEY, path);
    log('Extraction folder selected', { path });
  } catch (err) { /* logged */ }
});

$('importListfile').addEventListener('click', async () => {
  const path = $('listfilePath').value.trim();
  if (!path) return;
  try {
    const summary = await call('import_listfile', { path });
    state.listfileCount = summary.entries;
    $('metricListfile').textContent = summary.entries.toLocaleString();
    addQueue(`Indexed custom ListFile: ${summary.entries.toLocaleString()} entries`);
    log('Custom ListFile imported', summary);

    if (state.build?.path) {
      setCascStatus('Reloading CASC with the custom ListFile...');
      await openCascCatalog(state.build.path);
    }
  } catch (err) { /* logged */ }
});

async function searchIndex(query, target, selectable = false) {
  if (!query.trim()) return;
  const rows = await call('search_listfile', { query: query.trim(), limit: 250 });
  renderResults(rows, target, selectable);
  return rows;
}

$('searchFiles').addEventListener('click', () => searchIndex($('fileSearch').value, $('fileResults')).catch(() => {}));
$('fileSearch').addEventListener('keydown', (e) => { if (e.key === 'Enter') $('searchFiles').click(); });
$('addonSearchButton').addEventListener('click', () => searchIndex($('addonSearch').value, $('addonResults'), true).catch(() => {}));
$('addonSearch').addEventListener('keydown', (e) => { if (e.key === 'Enter') $('addonSearchButton').click(); });

function renderResults(rows, target, selectable) {
  target.classList.remove('empty-state');
  if (!rows.length) {
    target.innerHTML = '<div class="empty-state">No matches.</div>';
    return;
  }
  target.innerHTML = rows.map((r) => {
    const checked = state.luaSelected.has(String(r.file_data_id)) ? 'checked' : '';
    return `<div class="result-row"><span class="result-id">${r.file_data_id}</span><span class="result-path" title="${escapeAttr(r.path)}">${escapeHtml(r.path)}</span>${selectable ? `<input class="result-check" type="checkbox" data-fdid="${r.file_data_id}" data-path="${escapeAttr(r.path)}" ${checked}>` : '<span></span>'}</div>`;
  }).join('');
  if (selectable) {
    target.querySelectorAll('.result-check').forEach((box) => box.addEventListener('change', () => {
      const key = box.dataset.fdid;
      if (box.checked) state.luaSelected.set(key, { file_data_id: Number(key), path: box.dataset.path });
      else state.luaSelected.delete(key);
    }));
  }
}

$('inspectButton').addEventListener('click', async () => {
  const path = $('inspectPath').value.trim();
  if (!path) return;
  try {
    const info = await call('inspect_asset', { path });
    renderInspection(info, $('inspectResult'));
    log('Asset inspected', info);
  } catch (err) { /* logged */ }
});

$('db2InspectButton').addEventListener('click', async () => {
  const path = $('db2Path').value.trim();
  if (!path) return;
  try {
    const info = await call('inspect_asset', { path });
    renderInspection(info, $('db2Result'));
    log('Table inspected', info);
  } catch (err) { /* logged */ }
});

function renderInspection(info, target) {
  target.classList.remove('empty-state');
  const entries = Object.entries(info.details || {});
  target.innerHTML = `<dl class="inspect-grid"><dt>Format</dt><dd>${escapeHtml(info.format)}</dd><dt>Size</dt><dd>${Number(info.size).toLocaleString()} bytes</dd><dt>Path</dt><dd>${escapeHtml(info.path)}</dd>${entries.map(([k,v]) => `<dt>${escapeHtml(k)}</dt><dd>${escapeHtml(Array.isArray(v) ? v.join(', ') : String(v))}</dd>`).join('')}</dl>`;
}

$('decodeBlteButton').addEventListener('click', async () => {
  const input = $('inspectPath').value.trim();
  const output = $('blteOutputPath').value.trim();
  if (!input || !output) return;
  try {
    const result = await call('decode_blte_file', { inputPath: input, outputPath: output });
    addQueue(`Decoded BLTE -> ${output}`);
    log('BLTE decoded', result);
  } catch (err) { /* logged */ }
});

$('generateLua').addEventListener('click', () => {
  const items = [...state.luaSelected.values()].sort((a,b) => a.path.localeCompare(b.path));
  const used = new Map();
  const lines = ['-- Generated by WoW Ripper Addon Lab', '-- FileDataID constants', ''];
  for (const item of items) {
    const base = toLuaName(item.path);
    const count = (used.get(base) || 0) + 1;
    used.set(base, count);
    const name = count > 1 ? `${base}_${count}` : base;
    lines.push(`local ${name} = ${item.file_data_id} -- ${item.path}`);
  }
  $('luaOutput').value = lines.join('\n');
  log(`Generated ${items.length} Lua constant(s).`);
});

$('clearLuaSelection').addEventListener('click', () => {
  state.luaSelected.clear();
  $('luaOutput').value = '';
  $('addonResults').querySelectorAll('input[type="checkbox"]').forEach((b) => b.checked = false);
});

$('copyLua').addEventListener('click', async () => {
  const text = $('luaOutput').value;
  if (!text) return;
  await navigator.clipboard.writeText(text);
  log('Lua output copied to clipboard.');
});

function toLuaName(path) {
  const leaf = path.replace(/\\/g, '/').split('/').pop()?.replace(/\.[^.]+$/, '') || 'ASSET';
  let out = leaf.replace(/[^A-Za-z0-9]+/g, '_').replace(/^_+|_+$/g, '').toUpperCase();
  if (!out) out = 'ASSET';
  if (/^\d/.test(out)) out = `ASSET_${out}`;
  return `WR_${out}`;
}

function escapeHtml(value) { return String(value).replace(/[&<>"']/g, (c) => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#039;'}[c])); }
function escapeAttr(value) { return escapeHtml(value).replace(/`/g, '&#096;'); }

async function loadDefaultExtractRoot() {
  const remembered = localStorage.getItem(SAVED_EXTRACT_ROOT_KEY);
  if (remembered) {
    $('extractRoot').value = remembered;
    return;
  }
  if (!invoke) return;
  try {
    $('extractRoot').value = await call('default_extract_root');
  } catch (_) {}
}

updateBuildUI(null);
updateQueueUI();
loadDefaultExtractRoot();
loadRememberedBuild();
log('WoW Ripper HUD initialized. Drive scanning is disabled; WoW folder selection is explicit.');
