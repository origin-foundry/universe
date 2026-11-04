//! The SQLite project file: schema, migrations, repositories and cache management.
//!
//! A universe lives in a `.universe` file. While it's open, the app works on an in-memory copy
//! and the file is locked; nothing reaches the file until it's saved, and saving replaces the
//! file atomically. See `specs/003-project-file/contracts/store-and-commands.md`.

mod error;
mod name;
mod schema;
mod universe;

#[cfg(test)]
mod crash_safety_tests;

pub use error::StoreError;
pub use name::{MAX_NAME_CHARS, UniverseName};
pub use schema::{APPLICATION_ID, MIGRATIONS, SCHEMA_VERSION};
pub use universe::OpenUniverse;
