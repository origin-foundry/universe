# Research: Project File

## R1. SQLite access

- **Decision**: `rusqlite` in `uni-store`, with its `bundled` feature, so SQLite is compiled from source into the app. No system SQLite is needed, and every machine gets the same SQLite version. Use the current stable release, pinned in the build.
- **Rationale**: the verified storage concept (`/storage/schema.md`) and stack (`/architecture/stack.md`) already chose SQLite through `rusqlite`. Bundling avoids version differences between Windows and the Linux CI.
- **Alternatives**: `sqlx` (async, and heavier than needed for a desktop file).

## R2. Document model: the open universe lives in memory

- **Decision**: when a universe is opened, the file is read into an **in-memory SQLite database** using SQLite's backup API, and the app works on that copy. **Save** writes the in-memory database back out (R3). A new universe is an in-memory database that has never been saved.
- **Rationale**: this matches the owner's choice in Clarifications: nothing reaches the file until Save, and "Don't save" really discards. Later features edit the in-memory copy freely.
- **Alternatives**: editing the file directly inside a long transaction. Rejected: long-held write locks, and Save would be a commit rather than a file write, which loses Save As semantics.

## R3. Crash-safe save (FR-006, SC-004)

- **Decision**: Save backs up the in-memory database to a temporary file **in the same folder** (`<name>.universe.saving-<random>`), flushes it to disk, then atomically replaces the target with `std::fs::rename`, which on Windows is `MoveFileExW` with replace. If anything fails before the rename, the temporary file is deleted and the original is untouched. A test hook lets tests fail the save just before the rename, to prove this.
- **Rationale**: a rename within one folder is atomic on NTFS and on Linux file systems, so the file is always either the complete old one or the complete new one.
- **Alternatives**: writing in place inside a transaction (a crash can still leave a half-migrated file that has to be repaired from SQLite's journal; the temporary-file approach is simpler to reason about).

## R4. "File in use" (edge case)

- **Decision**: while a universe is open, `uni-store` holds an **exclusive `File::try_lock`** (stable in Rust 1.89+) on the project file. Opening a file that's already locked fails with `StoreError::InUse`. On Save, the lock moves to the new file after the rename.
- **Risk to check first in the build**: replacing a file that our own process holds open and locked. `std` opens files with `FILE_SHARE_DELETE` on Windows, so the rename should work. Task T004 proves it on Windows before anything else depends on it. If it doesn't work, the fallback is to release the lock, rename, then re-lock.
- **Alternatives**: a sidecar `.lock` file (leaves stray files next to the project after a crash).

## R5. Recognising our files, and versions (FR-003, FR-004, FR-008, FR-009)

- **Decision**:
  - **Identifier:** `PRAGMA application_id = 0x554E4956` (ASCII "UNIV"). Any other value, or a file that isn't SQLite, is `NotAUniverseFile`. A damaged file (SQLite reports it as "not a database" or "malformed") is also `NotAUniverseFile`, with the reason.
  - **Versions:** a `meta` table, as the storage concept says, holds `schema_version`, `app_version` (the app version that last wrote it) and `created_at`, as whole seconds since 1970 in UTC. That keeps it exact and avoids a date library.
  - **Newer files:** a `schema_version` above the app's is `NewerVersion { file, app }`.
  - **Older files:** run each migration step in order inside one transaction on the in-memory copy. First copy the original file to `<name>.v<old>.universe` beside it. The migrated copy reaches disk at the next Save, and the window shows unsaved changes.
- **Rationale**: `application_id` is SQLite's built-in way of marking file types. Keeping the version in `meta` follows the verified concept.

## R6. Schema version 1

- **Decision**: version 1 has only:
  - `meta(schema_version INTEGER, app_version TEXT, created_at INTEGER)`, one row;
  - `universe(id INTEGER PRIMARY KEY, name TEXT NOT NULL)`, one row.
  
  The other columns and tables in `/storage/schema.md` arrive with the features that use them, as numbered migrations (FR-005). Migrations are a `const` list of SQL steps in `uni-store`. Version 0, which has `meta` but no `universe` table, exists only as a test fixture, to exercise the migration path (spec Assumptions).

## R7. Names

- **Decision**: trim spaces at both ends; require 1 to 200 Unicode scalar values; keep the text exactly otherwise (no case or other normalisation). Validation lives in `uni-store` (`UniverseName::new`) so the UI and storage agree.

## R8. App integration (FR-002, FR-006b, FR-010, FR-011)

- **Decision**:
  - **State:** `uni-app` holds `Mutex<Option<OpenUniverse>>`, the one open universe, and exposes Tauri commands: `universe_state`, `new_universe(name)`, `open_universe(path)`, `save_universe()`, `save_universe_as(path)`, `close_universe(discard)`.
  - **File menu:** built in Rust with Tauri's menu API, with accelerators Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+Shift+S, Ctrl+W and Exit. Each item sends a `menu` event to the UI, which runs the same flow as the buttons. Save, Save As and Close are disabled while nothing is open.
  - **Dialogs:** `tauri-plugin-dialog` 2.x (the 3.x line is an alpha for Tauri 3) provides the native Open and Save dialogs, filtered to `*.universe`, and the "Save / Don't save / Cancel" message. Capabilities grant only `dialog:allow-open`, `dialog:allow-save` and `dialog:allow-message`.
  - **Closing the window** with unsaved changes is caught (`CloseRequested`), so the app can ask first.
  - **Title:** `Aster — Universe`, or `Aster • — Universe` with unsaved changes.
- **Rationale**: the menu and the buttons share one flow in the UI, so the behaviour is identical whichever the owner uses, and Vitest can test it with `invoke` mocked.

## R9. Tests

- **Decision**:
  - **`uni-store`** gets integration tests on temporary folders (`tempfile`, a dev-only dependency) for:
    - the round trip, including non-English names;
    - each kind of bad file (text, someone else's SQLite file, a truncated file, a newer version), each checked to be byte-for-byte unchanged;
    - migrating from version 0, with the backup copy kept;
    - an interrupted save, using the test hook, leaving the original intact;
    - a second open of the same file failing as in use;
    - save, save-as and the lock moving.
  - **`uni-app`:** a Rust test of the state transitions.
  - **UI:** Vitest tests of the empty screen, the new-universe form validation, the title and dirty marker, and the unsaved-changes prompt flow.
  - **Timing:** an `#[ignore]`d check that open plus save takes under 0.5 s (SC-002), run by hand.

## R10. Layering

- **Decision**: `uni-store` depends on no workspace crates yet (it doesn't need `uni-time` for whole-second timestamps), and on `rusqlite` only. `uni-app` gains its first workspace dependency: `uni-store`. Both are allowed by `/architecture/crate-layout.md`, so the knowledge concept doesn't change.
