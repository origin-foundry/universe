//! Performance check for SC-004: each conversion under 10 µs, near the epoch or 100,000 years
//! away (research R8).
//!
//! Ignored by default because timing on shared CI machines is noisy. Run it by hand:
//! `cargo test -p uni-calendar --release -- --ignored --nocapture`

mod common;

use std::hint::black_box;
use std::time::{Duration, Instant as Clock};

use common::*;
use uni_calendar::Calendar;
use uni_time::Instant;

const ROUNDS: i64 = 100_000; // per direction
/// About 100,000 Aster years, in sols.
const FAR: i64 = 100_000 * 668;

/// Half the instants near the epoch, half about 100,000 years away, a little apart.
fn instants() -> Vec<Instant> {
    (0..ROUNDS)
        .map(|i| {
            let base = if i % 2 == 0 { 0 } else { FAR };
            let step = ns(1_234_567_891 * i128::from(i));
            EPOCH
                .checked_add(sols(base))
                .and_then(|t| t.checked_add(step))
                .unwrap()
        })
        .collect()
}

#[test]
#[ignore = "timing check; run by hand in release mode"]
fn a_hundred_thousand_conversions_each_way_take_under_a_second() {
    let cal = Calendar::new(aster(*EPOCH)).unwrap();
    let instants = instants();
    let dates: Vec<_> = instants
        .iter()
        .map(|&t| cal.instant_to_date(t).unwrap().date)
        .collect();

    let start = Clock::now();
    for &t in &instants {
        black_box(cal.instant_to_date(black_box(t)).unwrap());
    }
    let to_date = start.elapsed();

    let start = Clock::now();
    for date in &dates {
        black_box(cal.date_to_instant(black_box(date)).unwrap());
    }
    let to_instant = start.elapsed();

    println!("{ROUNDS} instant_to_date in {to_date:?}; {ROUNDS} date_to_instant in {to_instant:?}");
    assert!(to_date < Duration::from_secs(1), "too slow: {to_date:?}");
    assert!(
        to_instant < Duration::from_secs(1),
        "too slow: {to_instant:?}"
    );
}
