# WoW Ripper

WoW Ripper is an original desktop research and extraction toolkit for World of Warcraft game data, built for addon authors, UI researchers, artists, and technical modding workflows.

This repository is a clean-room implementation. It does not contain source code copied from third-party WoW extractors.

## Goals

- Detect installed World of Warcraft products and builds.
- Read local build metadata and progressively support CASC/TACT storage directly.
- Index FileDataIDs and paths from community ListFiles.
- Inspect and export Blizzard asset formats used by World of Warcraft.
- Provide addon-focused research tools that turn asset discoveries into Lua-friendly data.
- Keep ripping, inspection, conversion, and queue management in one fast desktop UI.

## Alpha foundation

The initial foundation includes:

- Original Tauri/Rust desktop shell.
- WoW install/build scanner.
- `.build.info` parser.
- ListFile importer and in-memory FileDataID/path search.
- BLP2 metadata inspector.
- DB2/DBC signature and common-header inspector.
- BLTE decoder foundation for raw and zlib chunks.
- General asset magic inspector.
- Addon Lab with Lua constant generation.
- Extraction/activity queue HUD.

Full local CASC traversal, encrypted BLTE support, M2/WMO/ADT rendering, BLP conversion, DB2 schema decoding, and bulk extraction are tracked on the roadmap.

## Run from source

Prerequisites:

- Rust stable
- Platform requirements for Tauri 2

From `src-tauri`:

```bash
cargo tauri dev
```

The frontend intentionally uses plain HTML/CSS/JavaScript so the project has no Node frontend build dependency.

## Legal

WoW Ripper is an unofficial fan/developer tool and is not affiliated with or endorsed by Blizzard Entertainment. World of Warcraft and Blizzard-related names are trademarks of Blizzard Entertainment, Inc.

Use WoW Ripper only with game data you are legally permitted to access.

## License

MIT. See `LICENSE`.
