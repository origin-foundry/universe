//! An interrupted save leaves the previous file intact (spec 003, FR-006 and SC-004).

mod fixtures;

use fixtures::*;

#[test]
fn a_save_that_fails_before_the_rename_leaves_no_temporary_file() {
    let dir = tempfile::tempdir().unwrap();
    let (mut universe, _path) = saved(dir.path(), "aster.universe", "Aster");

    universe.fail_saves_before_rename(true);
    assert!(universe.save().is_err());
    assert_eq!(
        files_in(dir.path()),
        ["aster.universe"],
        "no temporary file is left"
    );

    universe.fail_saves_before_rename(false);
    universe.save().unwrap();
    assert!(!universe.is_dirty());
    assert_eq!(files_in(dir.path()), ["aster.universe"]);
}

#[test]
fn the_old_file_is_byte_for_byte_unchanged_after_a_failed_save() {
    let dir = tempfile::tempdir().unwrap();
    let (universe, path) = saved(dir.path(), "aster.universe", "Aster");
    drop(universe);
    let before = std::fs::read(&path).unwrap();

    let mut universe = uni_store::OpenUniverse::open(&path, APP_VERSION).unwrap();
    universe.fail_saves_before_rename(true);
    assert!(universe.save().is_err());
    assert!(
        universe
            .save_as(&dir.path().join("other.universe"))
            .is_err()
    );
    drop(universe);

    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(files_in(dir.path()), ["aster.universe"]);
}
