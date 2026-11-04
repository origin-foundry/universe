//! Helpers shared by the project-file tests: temporary folders, names, and files that aren't
//! current Universe project files.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use uni_store::{APPLICATION_ID, OpenUniverse, UniverseName};

pub const APP_VERSION: &str = "0.1.0-test";

pub fn name(raw: &str) -> UniverseName {
    UniverseName::new(raw).unwrap()
}

pub fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// A new universe called `raw`, saved to `dir/file`.
pub fn saved(dir: &Path, file: &str, raw: &str) -> (OpenUniverse, PathBuf) {
    let path = dir.join(file);
    let mut universe = OpenUniverse::new(name(raw), APP_VERSION).unwrap();
    universe.save_as(&path).unwrap();
    (universe, path)
}

/// Reads a project file's `PRAGMA application_id` and `meta` row directly.
pub fn read_meta(path: &Path) -> (i32, u32, String, i64) {
    let conn = Connection::open(path).unwrap();
    let id = conn
        .pragma_query_value(None, "application_id", |r| r.get(0))
        .unwrap();
    let (version, app, created) = conn
        .query_row(
            "SELECT schema_version, app_version, created_at FROM meta",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    (id, version, app, created)
}

/// A file that a text editor made and someone renamed.
pub fn text_file(path: &Path) {
    std::fs::write(path, "Not a database, just some notes.\n").unwrap();
}

/// A valid SQLite database that belongs to some other program.
pub fn foreign_sqlite(path: &Path) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch("CREATE TABLE notes (body TEXT); INSERT INTO notes VALUES ('hello');")
        .unwrap();
}

/// A version-0 project file: the identifier and `meta`, but no `universe` table.
pub fn v0_file(path: &Path) {
    let conn = Connection::open(path).unwrap();
    conn.pragma_update(None, "application_id", APPLICATION_ID)
        .unwrap();
    conn.execute_batch(
        "CREATE TABLE meta (schema_version INTEGER NOT NULL, app_version TEXT NOT NULL,
                            created_at INTEGER NOT NULL);
         INSERT INTO meta VALUES (0, '0.0.1', 1700000000);",
    )
    .unwrap();
}

/// Sets `meta.schema_version` in a saved file, to fake one from another version.
pub fn set_schema_version(path: &Path, version: u32) {
    let conn = Connection::open(path).unwrap();
    conn.execute("UPDATE meta SET schema_version = ?1", [version])
        .unwrap();
}

/// Cuts a file to half its length.
pub fn truncate_to_half(path: &Path) {
    let bytes = std::fs::read(path).unwrap();
    std::fs::write(path, &bytes[..bytes.len() / 2]).unwrap();
}

/// Names of the files in `dir`, sorted.
pub fn files_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}
