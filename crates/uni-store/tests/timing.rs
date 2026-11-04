//! SC-002: creating, saving and reopening an empty universe takes under 0.5 s in total.
//!
//! Run by hand, in release mode: `cargo test -p uni-store --release -- --ignored`.

mod fixtures;

use std::time::{Duration, Instant};

use fixtures::*;
use uni_store::OpenUniverse;

#[test]
#[ignore = "timing check; run by hand with --release -- --ignored"]
fn new_save_and_open_take_under_half_a_second() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("aster.universe");

    let start = Instant::now();
    let mut universe = OpenUniverse::new(name("Aster"), APP_VERSION).unwrap();
    universe.save_as(&path).unwrap();
    drop(universe);
    let reopened = OpenUniverse::open(&path, APP_VERSION).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(reopened.name().unwrap().as_str(), "Aster");
    println!("new + save_as + open: {elapsed:?}");
    assert!(elapsed < Duration::from_millis(500), "took {elapsed:?}");
}
