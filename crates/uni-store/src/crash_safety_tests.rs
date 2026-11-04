//! An interrupted save leaves the previous file intact (spec 003, FR-006 and SC-004).
//!
//! These are unit tests, not integration tests, because the hook that stops a save before
//! its rename exists only in `#[cfg(test)]` builds of this crate.

use std::path::{Path, PathBuf};

use crate::{OpenUniverse, UniverseName};

const APP_VERSION: &str = "0.1.0-test";

fn files_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn saved(dir: &Path) -> (OpenUniverse, PathBuf) {
    let path = dir.join("aster.universe");
    let mut universe = OpenUniverse::new(UniverseName::new("Aster").unwrap(), APP_VERSION).unwrap();
    universe.save_as(&path).unwrap();
    (universe, path)
}

#[test]
fn a_save_that_fails_before_the_rename_leaves_no_temporary_file() {
    let dir = tempfile::tempdir().unwrap();
    let (mut universe, _path) = saved(dir.path());

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
    let (universe, path) = saved(dir.path());
    drop(universe);
    let before = std::fs::read(&path).unwrap();

    let mut universe = OpenUniverse::open(&path, APP_VERSION).unwrap();
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
