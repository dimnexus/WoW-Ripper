# Clean-room development policy

WoW Ripper is developed as an original implementation.

## Rules

1. Third-party extractor source code is not imported into this repository.
2. Third-party UI layouts, branding, project structures, and source files are not used as implementation templates.
3. File-format behavior is implemented from public format documentation, observed file structures, and independently written tests/fixtures.
4. General-purpose dependencies may be used when their licenses are compatible with this project. Dependencies must be documented.
5. When a public specification is ambiguous, WoW Ripper should record the ambiguity in tests or documentation rather than copy another tool's implementation.
6. Blizzard trademarks and assets are not bundled as WoW Ripper branding.

## Why

The project exists so its architecture, implementation, history, and identity are independently maintainable. Git history should make that clear from the first functional commit onward.
