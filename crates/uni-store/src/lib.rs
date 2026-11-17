//! The SQLite project file: schema, migrations, repositories and cache management.
//!
//! A universe lives in a `.universe` file. While it's open, the app works on an in-memory copy
//! and the file is locked; nothing reaches the file until it's saved, and saving replaces the
//! file atomically. See `specs/003-project-file/contracts/store-and-commands.md`.
//!
//! # Calendars (spec 007, schema version 2)
//!
//! From schema version 2 a universe holds an ordered list of calendars, in the `calendar`,
//! `unit`, `unit_member`, `rule` and `era` tables. Each is kept as a [`CalendarRecord`]: the
//! definition exactly as the owner entered it, with the fundamental unit's length as the text
//! typed. [`CalendarRecord::to_def`] is the one path from a record to the engine's
//! `CalendarDef`, used both when the editor keeps a calendar and when a file is opened.
//!
//! The store checks calendar names but not definitions: the app checks a definition with the
//! engine before storing it, and a stored calendar the engine refuses can still be read back,
//! to be fixed or deleted. Version-1 files are migrated after a copy is kept beside them. See
//! `specs/007-calendars-in-project/contracts/calendar-commands.md`.

mod calendar;
mod error;
mod name;
mod schema;
mod universe;

#[cfg(test)]
mod crash_safety_tests;

pub use calendar::{CalendarRecord, EraRecord, FundamentalRecord, RecordError, StoredCalendar};
pub use error::StoreError;
pub use name::{MAX_NAME_CHARS, UniverseName};
pub use schema::{APPLICATION_ID, MIGRATIONS, SCHEMA_VERSION};
pub use universe::OpenUniverse;
