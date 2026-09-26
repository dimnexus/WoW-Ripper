# Changelog

## 0.1.0-alpha.3 - Windows desktop preview

- Added a native Windows folder picker for selecting WoW builds.
- Added a native extraction-output folder picker.
- Enabled NSIS Windows installer packaging.
- Added automated GitHub Release builds so normal users do not need Rust, Cargo, or PowerShell.
- Kept the alpha.2 live CASC folder/file browser, direct inspection, extraction, search, and Addon Lab integration.

## 0.1.0-alpha.2 - live CASC browser

- Added live CASC catalog opening for the explicitly selected WoW product.
- Added virtual folder and file browsing.
- Added direct FileDataID inspection from CASC.
- Added direct selected-file extraction with original virtual paths preserved.
- Added automatic catalog-backed Search and Addon Lab indexing.
- Added remembered extraction destination.
- Added optional custom ListFile override and catalog reload.
- Integrated the MIT-licensed casc-lib Rust transport dependency without vendoring third-party source.

# Changelog

## 0.1.0-alpha.1 - clean-room foundation

- Created an original Tauri/Rust desktop codebase.
- Added the first WoW Ripper HUD and workspace navigation.
- Added install/build discovery and `.build.info` parsing.
- Added ListFile FileDataID/path indexing and search.
- Added BLP2 and DB2/DBC metadata inspection.
- Added raw/zlib BLTE decoding foundation.
- Added Addon Lab Lua constant generation.
- Added clean-room development and architecture documentation.
- Added the original WR application icon required by the Windows desktop build.
