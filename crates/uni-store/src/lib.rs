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
//!
//! # Anchors (spec 008, schema version 3)
//!
//! From schema version 3 each calendar may have an [`Anchor`], in the `anchor` table, that
//! places it in time. An anchor places its **anchored calendar** from a **base calendar**: its
//! **anchor date** starts at the same moment as the **base date** in the base calendar. It
//! either fixes the moment (the instant is worked out when the anchor is kept, and stored) or
//! keeps the equation (the anchored calendar **follows** its base, and moves when the base is
//! edited). The one calendar without an anchor, if there is one, is the **reference
//! calendar**, at the hidden zero; that's a term for the code, which the app doesn't show.
//!
//! Epochs are never stored. [`OpenUniverse::placements`] works out where every calendar is,
//! or why it can't be placed; [`OpenUniverse::check_anchor`] checks an anchor before the app
//! keeps a calendar; deleting a calendar fixes its followers where they stand. Version-2 files
//! are migrated with every later calendar anchored to the first at their first dates, so
//! nothing moves. See `specs/008-epoch-anchoring/contracts/anchors-and-converter.md`.

mod anchor;
mod calendar;
mod error;
mod name;
mod schema;
mod universe;

#[cfg(test)]
mod crash_safety_tests;

pub use anchor::{
    Anchor, AnchorError, AnchorKind, AnchorSide, Placement, PlacementProblem, follows,
};
pub use calendar::{CalendarRecord, EraRecord, FundamentalRecord, RecordError, StoredCalendar};
pub use error::StoreError;
pub use name::{MAX_NAME_CHARS, UniverseName};
pub use schema::{APPLICATION_ID, MIGRATIONS, SCHEMA_VERSION};
pub use universe::OpenUniverse;
