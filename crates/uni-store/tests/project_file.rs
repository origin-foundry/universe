//! Creating, saving and opening project files (spec 003, user stories 1 and 2).

mod fixtures;

use fixtures::*;
use uni_store::{APPLICATION_ID, OpenUniverse, StoreError};

// ---- User story 1: create and save ----

#[test]
fn new_gives_a_dirty_universe_with_no_path() {
    let universe = OpenUniverse::new(name("Aster"), APP_VERSION).unwrap();
    assert!(universe.is_dirty());
    assert_eq!(universe.path(), None);
    assert_eq!(universe.name().unwrap().as_str(), "Aster");
}

#[test]
fn save_without_a_path_is_refused_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let mut universe = OpenUniverse::new(name("Aster"), APP_VERSION).unwrap();
    assert_eq!(universe.save(), Err(StoreError::NoPath));
    assert!(universe.is_dirty());
    assert!(files_in(dir.path()).is_empty());
}

#[test]
fn save_as_writes_a_marked_file_and_the_universe_becomes_clean() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("aster.universe");
    let mut universe = OpenUniverse::new(name("Aster"), APP_VERSION).unwrap();
    universe.save_as(&path).unwrap();

    assert!(!universe.is_dirty());
    assert_eq!(universe.path(), Some(path.as_path()));
    drop(universe); // release the lock so the file can be read directly

    let (id, version, app, created) = read_meta(&path);
    assert_eq!(id, APPLICATION_ID);
    assert_eq!(version, 1);
    assert_eq!(app, APP_VERSION);
    assert!(
        (now_unix() - created).abs() <= 5,
        "created_at {created} is not now"
    );
    let conn = rusqlite::Connection::open(&path).unwrap();
    let stored: String = conn
        .query_row("SELECT name FROM universe WHERE id = 1", [], |r| r.get(0))
        .unwrap();
    assert_eq!(stored, "Aster");
    assert_eq!(files_in(dir.path()), ["aster.universe"]);
}

#[test]
fn saving_again_keeps_created_at() {
    let dir = tempfile::tempdir().unwrap();
    let (mut universe, path) = saved(dir.path(), "aster.universe", "Aster");
    let created = universe.created_at_unix().unwrap();
    universe.save().unwrap();
    assert_eq!(universe.created_at_unix().unwrap(), created);
    drop(universe);
    assert_eq!(read_meta(&path).3, created);
    assert_eq!(files_in(dir.path()), ["aster.universe"]);
}

#[test]
fn saving_into_a_missing_folder_is_an_io_error_and_the_universe_stays_open() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("no such folder").join("aster.universe");
    let mut universe = OpenUniverse::new(name("Aster"), APP_VERSION).unwrap();
    let error = universe.save_as(&path).unwrap_err();
    assert!(matches!(error, StoreError::Io { .. }), "{error:?}");
    assert!(universe.is_dirty());
    assert_eq!(universe.path(), None);
    assert_eq!(universe.name().unwrap().as_str(), "Aster");
}

#[test]
fn saving_after_the_file_was_deleted_is_missing_and_save_as_still_works() {
    let dir = tempfile::tempdir().unwrap();
    let (mut universe, path) = saved(dir.path(), "aster.universe", "Aster");
    std::fs::remove_file(&path).unwrap();

    assert_eq!(
        universe.save(),
        Err(StoreError::Missing { path: path.clone() })
    );
    assert!(
        files_in(dir.path()).is_empty(),
        "save must not recreate the file"
    );

    let elsewhere = dir.path().join("rescued.universe");
    universe.save_as(&elsewhere).unwrap();
    assert_eq!(universe.path(), Some(elsewhere.as_path()));
}

#[test]
fn saving_after_the_file_was_moved_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let (mut universe, path) = saved(dir.path(), "aster.universe", "Aster");
    let moved = dir.path().join("moved.universe");
    std::fs::rename(&path, &moved).unwrap();

    assert_eq!(
        universe.save(),
        Err(StoreError::Missing { path: path.clone() })
    );
    assert_eq!(files_in(dir.path()), ["moved.universe"]);
}

#[test]
fn save_as_moves_the_lock_to_the_new_file() {
    let dir = tempfile::tempdir().unwrap();
    let (mut universe, first) = saved(dir.path(), "first.universe", "Aster");
    let second = dir.path().join("second.universe");
    universe.save_as(&second).unwrap();

    // The first file is free again; the second is still held.
    OpenUniverse::open(&first, APP_VERSION).unwrap();
    assert_eq!(
        OpenUniverse::open(&second, APP_VERSION).err(),
        Some(StoreError::InUse)
    );
}

#[test]
fn save_as_over_a_file_open_elsewhere_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let (_other, taken) = saved(dir.path(), "taken.universe", "Other");
    let mut universe = OpenUniverse::new(name("Aster"), APP_VERSION).unwrap();
    assert_eq!(universe.save_as(&taken), Err(StoreError::InUse));
    assert!(universe.is_dirty());
}

// ---- User story 2: open ----

#[test]
fn round_trip_keeps_the_name_exactly() {
    let dir = tempfile::tempdir().unwrap();
    let (universe, path) = saved(dir.path(), "aster.universe", "Aster ✶ Ñandú");
    drop(universe);

    let reopened = OpenUniverse::open(&path, APP_VERSION).unwrap();
    assert_eq!(reopened.name().unwrap().as_str(), "Aster ✶ Ñandú");
    assert!(!reopened.is_dirty());
    assert_eq!(reopened.path(), Some(path.as_path()));
}

fn assert_refused_and_unchanged(path: &std::path::Path, expected: impl Fn(&StoreError) -> bool) {
    let before = std::fs::read(path).unwrap();
    let error = OpenUniverse::open(path, APP_VERSION)
        .err()
        .expect("must be refused");
    assert!(expected(&error), "unexpected error {error:?}");
    assert_eq!(
        std::fs::read(path).unwrap(),
        before,
        "the file must be unchanged"
    );
}

fn not_a_universe_file(error: &StoreError) -> bool {
    matches!(error, StoreError::NotAUniverseFile { .. })
}

#[test]
fn a_renamed_text_file_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.universe");
    text_file(&path);
    assert_refused_and_unchanged(&path, not_a_universe_file);
}

#[test]
fn an_empty_file_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.universe");
    std::fs::write(&path, b"").unwrap();
    assert_refused_and_unchanged(&path, not_a_universe_file);
}

#[test]
fn another_programs_sqlite_file_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("foreign.universe");
    foreign_sqlite(&path);
    assert_refused_and_unchanged(&path, not_a_universe_file);
}

#[test]
fn a_truncated_file_is_refused_as_damaged() {
    let dir = tempfile::tempdir().unwrap();
    let (universe, path) = saved(dir.path(), "aster.universe", "Aster");
    drop(universe);
    truncate_to_half(&path);
    assert_refused_and_unchanged(&path, |e| {
        *e == StoreError::NotAUniverseFile {
            reason: "it's damaged".into(),
        }
    });
}

#[test]
fn a_file_from_a_newer_version_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let (universe, path) = saved(dir.path(), "aster.universe", "Aster");
    drop(universe);
    set_schema_version(&path, 2);
    assert_refused_and_unchanged(&path, |e| {
        *e == StoreError::NewerVersion { file: 2, app: 1 }
    });
    assert_eq!(files_in(dir.path()), ["aster.universe"]);
}

#[test]
fn an_older_file_is_migrated_after_keeping_a_copy() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("old.universe");
    v0_file(&path);
    let original = std::fs::read(&path).unwrap();

    let universe = OpenUniverse::open(&path, APP_VERSION).unwrap();
    assert_eq!(universe.name().unwrap().as_str(), "Untitled");
    assert!(
        universe.is_dirty(),
        "a migrated universe has unsaved changes"
    );
    drop(universe);

    assert_eq!(files_in(dir.path()), ["old.universe", "old.v0.universe"]);
    assert_eq!(
        std::fs::read(dir.path().join("old.v0.universe")).unwrap(),
        original
    );
    assert_eq!(
        std::fs::read(&path).unwrap(),
        original,
        "not rewritten until saved"
    );
}

#[test]
fn a_file_already_open_is_in_use_until_closed() {
    let dir = tempfile::tempdir().unwrap();
    let (universe, path) = saved(dir.path(), "aster.universe", "Aster");
    drop(universe);

    let first = OpenUniverse::open(&path, APP_VERSION).unwrap();
    assert_eq!(
        OpenUniverse::open(&path, APP_VERSION).err(),
        Some(StoreError::InUse)
    );
    drop(first);
    OpenUniverse::open(&path, APP_VERSION).unwrap();
}

#[test]
fn opening_a_file_that_does_not_exist_is_an_io_error() {
    let dir = tempfile::tempdir().unwrap();
    let error = OpenUniverse::open(&dir.path().join("nope.universe"), APP_VERSION)
        .err()
        .unwrap();
    assert!(
        matches!(error, StoreError::Io { action: "open", .. }),
        "{error:?}"
    );
}
