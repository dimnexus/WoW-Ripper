# Architecture

WoW Ripper is split into a thin desktop shell and an original Rust data core.

## Frontend

src/

- Plain HTML/CSS/JavaScript.
- No framework lock-in.
- Talks to Rust only through explicit Tauri commands.
- Owns presentation state, navigation, queue visualization, and Lua snippet generation.

## Rust core

src-tauri/src/core/

- installs.rs - discovers likely WoW installation roots.
- build_info.rs - parses Battle.net .build.info metadata generically.
- listfile.rs - imports FileDataID/path indexes and provides fast search.
- blp.rs - BLP metadata parsing.
- db2.rs - DBC/DB2 metadata parsing.
- blte.rs - BLTE container decoding.
- inspector.rs - format identification and unified inspection result.
- types.rs - serializable API types shared by commands.

## Boundary rule

The frontend never parses Blizzard binary formats. The Rust core never depends on DOM/UI details. This lets future CLI, automation, or server tools reuse the same format core.

## CASC plan

The local CASC implementation will be layered rather than monolithic:

1. Battle.net build metadata
2. Local .idx lookup
3. Data archive block reads
4. BLTE decode
5. Encoding table mapping
6. Root mapping
7. Product-specific FileDataID/path resolution

Each layer will have independent fixtures and tests.
