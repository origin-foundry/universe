# Contract: `uni-store` API and the app's commands

## `uni-store` (Rust)

```rust
pub const APPLICATION_ID: i32 = 0x554E4956;
pub const SCHEMA_VERSION: u32 = 1;

pub struct UniverseName(String);
impl UniverseName {
    pub fn new(raw: &str) -> Result<UniverseName, StoreError>;   // trims; 1..=200 chars
    pub fn as_str(&self) -> &str;
}

pub struct OpenUniverse { /* in-memory db, path, dirty, lock */ }
impl OpenUniverse {
    pub fn new(name: UniverseName, app_version: &str) -> Result<OpenUniverse, StoreError>;
    pub fn open(path: &Path, app_version: &str) -> Result<OpenUniverse, StoreError>; // locks; migrates older
    pub fn save(&mut self) -> Result<(), StoreError>;                 // NoPath if never saved; Missing if moved/deleted
    pub fn save_as(&mut self, path: &Path) -> Result<(), StoreError>; // moves the lock
    pub fn name(&self) -> Result<UniverseName, StoreError>;
    pub fn path(&self) -> Option<&Path>;
    pub fn is_dirty(&self) -> bool;
    pub fn created_at_unix(&self) -> Result<i64, StoreError>;
}
// Dropping an OpenUniverse releases its lock.

pub enum StoreError { InvalidName, NotAUniverseFile { reason: String },
    NewerVersion { file: u32, app: u32 }, InUse, NoPath, Missing { path: PathBuf },
    Io { action: &'static str, path: PathBuf, reason: String }, Database { reason: String } }
// Display gives the owner-facing messages in data-model.md; implements std::error::Error.
```

A `#[cfg(test)]`-only hook makes `save`/`save_as` fail just before the rename (R3).

## Tauri commands (`uni-app`)

All return `UniverseState` on success, or a string message (the `StoreError` text) on failure.

| Command | Arguments | Effect |
|---|---|---|
| `universe_state` | none | Current state |
| `new_universe` | `name: String` | Replaces any open universe (the UI has already asked about unsaved changes) |
| `open_universe` | `path: String` | Same, from a file |
| `save_universe` | none | `NoPath` if untitled, `Missing` if the file was moved or deleted; the UI then runs Save As |
| `save_universe_as` | `path: String` | Saves and continues in the new file |
| `close_universe` | none | Back to no universe |

```ts
interface UniverseState { open: boolean; name: string | null; path: string | null; dirty: boolean; }
```

## Events (Rust → UI)

- `menu`, with payload `"new" | "open" | "save" | "save-as" | "close"`, when a File menu item or its shortcut is used. Exit goes through the window-close flow.
- `close-requested`: the window was asked to close. The UI runs the unsaved-changes prompt, then calls `close_universe` and asks the window to close.
