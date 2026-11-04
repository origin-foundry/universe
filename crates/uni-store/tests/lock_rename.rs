//! Spike for research.md R4: can a file our own process holds open and locked be replaced by a
//! rename? `OpenUniverse` relies on this to keep its lock across a save.

use std::fs::{self, File, OpenOptions};
use std::io::Write;

#[test]
fn a_locked_file_can_be_replaced_by_rename_and_relocked() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("aster.universe");
    let temp = dir.path().join("aster.universe.saving-test");

    fs::write(&target, b"old").unwrap();
    let held = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&target)
        .unwrap();
    held.try_lock().expect("the first lock succeeds");

    let mut replacement = File::create(&temp).unwrap();
    replacement.write_all(b"new").unwrap();
    replacement.sync_all().unwrap();
    drop(replacement);

    fs::rename(&temp, &target).expect("rename over a file we hold locked");
    drop(held);

    assert_eq!(fs::read(&target).unwrap(), b"new");
    let reopened = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&target)
        .unwrap();
    reopened.try_lock().expect("the new file can be locked");
}
