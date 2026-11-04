---
description: "Task list for spec 003, project file"
---

# Tasks: Project File

**Input**: Design documents from `specs/003-project-file/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/store-and-commands.md](contracts/store-and-commands.md), [quickstart.md](quickstart.md)

**Tests**: Included and written first (constitution VI). Each test task must fail before its implementation task. Tasks marked **(owner)** are done by `origin-foundry`.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [X] T001 In `crates/uni-store/Cargo.toml`, add `rusqlite` (current stable, `features = ["bundled"]`) and the dev-dependency `tempfile`.
- [X] T002 In `crates/uni-app/Cargo.toml`, add `uni-store = { path = "../uni-store" }` and `tauri-plugin-dialog` (2.x). In `package.json`, add `@tauri-apps/plugin-dialog` (2.x). In `crates/uni-app/capabilities/default.json`, add `dialog:allow-open`, `dialog:allow-save` and `dialog:allow-message`. Run `python scripts/check_architecture.py` to confirm the new edge is allowed.

---

## Phase 2: Foundational

- [X] T003 [P] Write `crates/uni-store/src/error.rs`: `StoreError` per the contract, with `Display` messages exactly as in data-model.md, `std::error::Error`, and `From<rusqlite::Error>` mapping "not a database" or "malformed" to `NotAUniverseFile` and everything else to `Database`. Include a unit test of every message.
- [X] T004 Spike the lock risk (research.md R4) in a test, `crates/uni-store/tests/lock_rename.rs`. Create a file, `File::try_lock` it, then write a second file and `std::fs::rename` it over the first while the lock is held. Assert the rename succeeds and the new file can be opened and locked. It must pass on Windows. If it fails, switch `OpenUniverse` to release, rename, then re-lock, and say so in the build PR.
- [X] T005 [P] Write `crates/uni-store/src/name.rs`: `UniverseName::new` trims, and accepts 1 to 200 Unicode scalar values. Unit tests: `""`, `"   "` and 201 characters are rejected; 200 characters, `"  Aster  "` (stored as `"Aster"`) and `"Aster ✶ Ñandú"` are accepted.
- [X] T006 Write `crates/uni-store/src/schema.rs`: `APPLICATION_ID = 0x554E4956`, `SCHEMA_VERSION = 1`, and a `MIGRATIONS` list where step 0→1 creates `universe(id INTEGER PRIMARY KEY, name TEXT NOT NULL)` with `name = 'Untitled'`. Add `create_v1(conn, name, app_version, now_unix)` and `migrate(conn, from)` running steps in one transaction. Write `crates/uni-store/src/lib.rs` re-exporting the public API.

**Checkpoint**: `cargo build -p uni-store` and `cargo clippy -p uni-store --all-targets -- -D warnings` pass.

---

## Phase 3: User Story 1 - Create and save a new universe (Priority: P1)

### Tests (write first; they must fail)

- [X] T007 [US1] Write `crates/uni-store/tests/fixtures.rs` (helpers) and, in `crates/uni-store/tests/project_file.rs`, these cases:
  - `new` gives a dirty universe with no path, whose name is the one given;
  - `save` with no path returns `NoPath`, and nothing is written;
  - `save_as` writes a file with `application_id` 0x554E4956, a `meta` row (`schema_version` 1, the app version, `created_at` within 5 s of now) and the `universe` row; the universe becomes clean and its `path` is set;
  - a second `save` to the same path succeeds, and `created_at` is unchanged;
  - saving into a folder that doesn't exist gives an `Io` error, and the universe stays open and dirty;
  - after a successful save, deleting or moving the file and then calling `save` gives `Missing { path }`, no new file is created, and `save_as` to a new path still works.
- [X] T008 [US1] Write `crates/uni-store/tests/crash_safety.rs`:
  - save once; then enable the test hook so the next save fails just before the rename; that save returns an error;
  - the file is byte-for-byte the old version, and no `*.saving-*` temporary file is left behind;
  - with the hook off, a save then succeeds.

### Implementation

- [X] T009 [US1] Implement `OpenUniverse::new`, `save`, `save_as`, `name`, `path`, `is_dirty` and `created_at_unix` in `crates/uni-store/src/universe.rs`, using an in-memory database, the backup API to a same-folder temporary file, `sync_all`, then `rename` (research.md R2 and R3). Before each `save`, check the file is still at its path (and is the locked one); if not, return `Missing`. The lock is taken after the first successful save and moved on `save_as`. Add a `#[cfg(test)]`-only, or test-feature-gated, hook to fail before the rename. Work until T007 and T008 pass.

**Checkpoint**: a universe can be created and saved from Rust.

---

## Phase 4: User Story 2 - Open an existing universe (Priority: P1)

### Tests (write first; they must fail)

- [X] T010 [US2] Extend `crates/uni-store/tests/project_file.rs`:
  - **round trip:** save `"Aster ✶ Ñandú"`, drop it, open it: the name is identical, and the universe is clean.
  - **bad files**, each refused, with the file's bytes unchanged afterwards:
    - a text file renamed `.universe` → `NotAUniverseFile`;
    - a valid SQLite file with `application_id` 0 → `NotAUniverseFile`;
    - a saved file truncated to half its length → `NotAUniverseFile` (damaged);
    - a file whose `meta.schema_version` is set to 2 → `NewerVersion { file: 2, app: 1 }`.
  - **older version:** a fixture v0 file opens and is migrated, giving the name `Untitled`, a dirty universe, and `<name>.v0.universe` beside it, byte-identical to the original fixture. The original isn't rewritten until saved.
  - **in use:** opening the same path twice while the first is still open → `InUse`; after dropping the first, opening works.

### Implementation

- [X] T011 [US2] Implement `OpenUniverse::open` in `crates/uni-store/src/universe.rs`: lock first (`InUse` on failure); read through a read-only connection; check `application_id`, then `meta`; refuse newer versions; for older ones, copy the file to `<stem>.v<old>.universe` and then migrate the in-memory copy; load it into memory with the backup API. Work until T010 passes.
- [X] T012 [P] [US2] Write `crates/uni-store/tests/timing.rs`, an `#[ignore]` check that `new` + `save_as` + `open` takes under 0.5 s in total (SC-002). Run it with `cargo test -p uni-store --release -- --ignored` and record the result in the build PR description.

**Checkpoint**: the store side of the round trip is complete.

---

## Phase 5: User Story 3 - Close a universe, and the app (US1–US3 in the UI)

### Tests (write first; they must fail)

- [ ] T013 [P] [US3] In `crates/uni-app/src/lib.rs`, write a `#[cfg(test)]` test of the state holder, with no Tauri runtime:
  - new → state `open: true, dirty: true, path: None`;
  - `save_as` to a temp path → clean, with the path set;
  - close → `open: false`;
  - open → the name matches.
- [ ] T014 [P] [US3] Write `ui/src/fileFlows.test.ts`, with `invoke` and the dialog plugin mocked:
  - New with a clean universe open asks nothing;
  - New with a dirty one shows Save / Don't save / Cancel, and Cancel stops;
  - Save on an untitled universe opens the Save dialog with the universe's name as the suggested file name and the `.universe` filter;
  - a cancelled dialog changes nothing;
  - a `StoreError` message from `invoke` is shown to the owner;
  - when Save fails with the `Missing` message, the flow offers Save As.
- [ ] T015 [P] [US3] Update `ui/src/App.test.tsx` and write `ui/src/NewUniverseForm.test.tsx`:
  - the empty screen shows "No universe open" and **New universe** and **Open universe…** buttons;
  - the form rejects empty and too-long names with the message from data-model.md;
  - an open universe shows its name, and the document title is `Aster — Universe`, or `Aster • — Universe` when dirty.

### Implementation

- [ ] T016 [US3] In `crates/uni-app/src/lib.rs`:
  - add the `Mutex<Option<OpenUniverse>>` state and the six commands from the contract;
  - add the File menu (New Ctrl+N, Open Ctrl+O, Save Ctrl+S, Save As Ctrl+Shift+S, Close Ctrl+W, Exit), emitting `menu` events, with Save, Save As and Close enabled only while a universe is open;
  - register the dialog plugin;
  - turn `CloseRequested` into a `close-requested` event, and let the window close once the UI has finished.
  
  Work until T013 passes.
- [ ] T017 [US3] Implement `ui/src/fileFlows.ts`, `ui/src/NewUniverseForm.tsx` and the `App.tsx` changes: listen for `menu` and `close-requested` events; keep the document title in step with the state. Work until T014 and T015 pass.
- [ ] T018 [US3] **(owner)** Run quickstart.md steps 4 and 5 with `npm run dev` and report the time for steps a to f (SC-001: under 1 minute) and anything unexpected.

---

## Phase 6: Polish

- [ ] T019 Add doc comments with a short example to `UniverseName` and `OpenUniverse`, and a "Project files" section to `docs/development.md` covering where files go, what the `.v<n>.universe` copies are, and that they can be deleted once you're happy with the migrated file.
- [ ] T020 Run the full local verification and record the results in the build PR description:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `cargo deny check advisories bans sources`
  - `npm run typecheck`, `npm run lint`, `npm run format:check`, `npm test`, `npm audit --audit-level=high`
  - `python scripts/check_architecture.py`
  - `python -m unittest discover -s scripts/tests`
  - `python scripts/check_okf.py knowledge`
- [ ] T021 Open the build PR as the bot: label `build`, body with `Design: #<design PR>` and `Closes #11`, and a review order of store, then app, then UI (plan.md Complexity Tracking). Move card #11 to **PR review**.

---

## Dependencies & Execution Order

- **Setup → Foundational**: T001 and T002, then T003 to T006. T004 (the lock spike) must pass before T009.
- **US1** (T007 to T009) → **US2** (T010 to T012): opening needs saved files to open.
- **US3** (T013 to T017) needs US1 and US2's store API. T013, T014 and T015 can be written in parallel.
- **T018** (owner) needs everything above. **Polish** comes last.

## Notes

- Tick each task as it's done; the PR policy refuses a build PR with unticked tasks.
- Build PRs may only change `tasks.md` within `specs/`. A design change found during the build (for example, if T004 fails) gets noted in the PR; a change to the spec or knowledge needs a design PR.
