# Implementation Plan: Project File

**Branch**: `design/003-project-file` (design), `build/003-project-file` (build) | **Date**: 2026-11-04 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/003-project-file/spec.md` (approved by the owner, 2026-11-04)

## Summary

- **`uni-store`:** gains the project file. A `.universe` SQLite file is marked with an application identifier and a `meta` table, and is read into an in-memory database on open. It's saved crash-safely through a temporary file and a rename, and locked while open. Files that aren't ours or come from a newer version are refused; older files are migrated after a backup copy is kept.
- **`uni-app`:** holds the one open universe and exposes commands for it. It also adds a native File menu with shortcuts, native dialogs, and handles closing the window.
- **The UI:** shows the empty screen's New and Open buttons, a name form, the universe's name and the unsaved-changes prompt.

Details are in [research.md](research.md).

## Technical Context

**Language/Version**: Rust 1.99.1, edition 2024; TypeScript (strict).

**Primary Dependencies**:
- `rusqlite` (with SQLite bundled) in `uni-store`.
- `tauri-plugin-dialog` 2.x and `@tauri-apps/plugin-dialog` 2.x in the app.
- `tempfile` (dev-only).

**Storage**: SQLite `.universe` files, schema version 1 ([data-model.md](data-model.md)).

**Testing**: `cargo test -p uni-store` (integration tests in temporary folders), a Rust state test in `uni-app`, Vitest for the UI flows, a timing check run by hand, and an owner trial.

**Target Platform**: Windows for the app; `uni-store` also tested on Linux.

**Project Type**: desktop app feature across a library crate, the app crate and the UI.

**Performance Goals**: open plus save under 0.5 s for an empty universe (SC-002).

**Constraints**:
- Crash-safe saves.
- Nothing written without the owner saving.
- Least-privilege Tauri capabilities: dialog `open`, `save` and `message` only.
- `uni-store` independent of Tauri.

**Scale/Scope**: three areas (store, app, UI); about 900 lines including tests.

## Constitution Check

*GATE: passed before research and again after design.*

| Principle | How this plan complies |
|---|---|
| I. Spec before code | Approved spec; this design PR, then a build PR. |
| II. Knowledge is the source of truth | Follows `/storage/schema.md`. That concept gains the application identifier, versioning and migrations, crash-safe saving and locking, so it goes back to draft for re-verification. |
| III. Exact calendar arithmetic | Not affected; `uni-store` isn't an exact crate, and timestamps are whole seconds. |
| IV. Hidden absolute timeline | Not affected; no time is shown. |
| V. Layering | `uni-store` has no workspace dependencies; `uni-app → uni-store` is allowed. Only `uni-app` uses Tauri. |
| VI. Test-first | Store and UI tests are written before the code; bad-file and interrupted-save cases are tested directly. |
| VII. No silent ambiguity | Every failure is a named `StoreError` with a message; nothing is overwritten or migrated without a copy, and unsaved work always prompts. |
| VIII. Small changes | About 900 lines across three areas. That's at the top of a 30-minute review; see Complexity Tracking. |

## Project Structure

### Documentation (this feature)

```text
specs/003-project-file/
├── spec.md, plan.md, research.md, data-model.md, quickstart.md
├── contracts/store-and-commands.md
├── checklists/requirements.md
└── tasks.md
```

### Source Code

```text
crates/uni-store/
├── Cargo.toml            # rusqlite (bundled); dev: tempfile
├── src/lib.rs            # re-exports
├── src/error.rs          # StoreError
├── src/name.rs           # UniverseName
├── src/schema.rs         # APPLICATION_ID, SCHEMA_VERSION, migrations
├── src/universe.rs       # OpenUniverse: new, open, save, save_as, lock
└── tests/
    ├── fixtures.rs       # helpers: bad files, a v0 file
    ├── project_file.rs   # round trip, bad files, migration, in-use, save-as
    ├── crash_safety.rs   # interrupted save
    └── timing.rs         # #[ignore] SC-002
crates/uni-app/
├── Cargo.toml            # + uni-store, tauri-plugin-dialog
├── src/lib.rs            # state, commands, menu, close handling
└── capabilities/default.json   # + dialog:allow-open/save/message
ui/src/
├── App.tsx               # empty screen / open universe; menu + close events
├── NewUniverseForm.tsx   # name entry with validation
├── fileFlows.ts          # new/open/save/save-as/close flows + unsaved prompt
└── *.test.tsx / *.test.ts
package.json              # + @tauri-apps/plugin-dialog
```

**Changed in this design PR**: `knowledge/storage/schema.md`, `knowledge/log.md`.

## Complexity Tracking

| Concern | Why it's needed | Simpler alternative rejected because |
|---|---|---|
| About 900 lines across store, app and UI | The spec's round trip only works end to end with all three | Splitting store from UI leaves an unusable half and two reviews of the same flows. Instead, the build PR's description gives a review order: store, then app, then UI. |
