# Data Model: Project File

## The file (schema version 1)

| Item | Value / columns | Rule |
|---|---|---|
| `PRAGMA application_id` | `0x554E4956` ("UNIV") | Any other value: not a Universe file |
| `meta` | `schema_version INTEGER NOT NULL`, `app_version TEXT NOT NULL`, `created_at INTEGER NOT NULL` | Exactly one row. `created_at` is whole seconds since 1970-01-01 UTC and never changes; `app_version` is rewritten on every save. |
| `universe` | `id INTEGER PRIMARY KEY`, `name TEXT NOT NULL` | Exactly one row, `id = 1` |

Version 0 (test fixture only): `application_id` set and `meta` present with `schema_version = 0`, but no `universe` table. The v0 → v1 migration creates `universe` with the name `Untitled`.

## In the app

| Type | Fields | Rules |
|---|---|---|
| `UniverseName` | the text | Trimmed; 1 to 200 Unicode scalar values; otherwise unchanged |
| `OpenUniverse` | in-memory database; `path: Option<PathBuf>` (None until first saved); `dirty: bool`; the file lock (when it has a path) | `dirty` is true for a new universe and after a migration; cleared by a successful save |
| `UniverseState` (sent to the UI) | `open: bool`, `name`, `path`, `dirty` | |

## StoreError

| Variant | When | Message (shown to the owner) |
|---|---|---|
| `InvalidName` | Empty or too long | "A universe needs a name of 1 to 200 characters." |
| `NotAUniverseFile { reason }` | Not SQLite, a different `application_id`, or damaged | "This isn't a Universe project file ({reason})." |
| `NewerVersion { file, app }` | `schema_version` above the app's | "This file was saved by a newer version of Universe (file version {file}, this app reads up to {app})." |
| `InUse` | The lock is held elsewhere | "This file is open in another window or program." |
| `NoPath` | Save on a universe that has never been saved | "This universe hasn't been saved yet. Use Save As." |
| `Missing { path }` | The file was moved or deleted while open | "{path} has been moved or deleted. Use Save As to save it somewhere." |
| `Io { action, path, reason }` | Read, write or rename failures | "Couldn't {action} {path}: {reason}." |
| `Database { reason }` | Unexpected SQLite errors | "Something went wrong with the project file: {reason}." |

## State transitions

```
none --new(name)--> open(untitled, dirty)
none --open(path)--> open(path, clean) | open(path, dirty) if migrated
open --save--> open(path, clean)          [untitled: asks for a path first]
open --save_as(path)--> open(new path, clean)   [lock moves]
open --close(discard or clean)--> none
open --new/open--> (unsaved-changes prompt if dirty) --> new state
```
