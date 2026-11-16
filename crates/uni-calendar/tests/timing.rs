//! Performance checks for SC-004 (specs 004 and 006): each conversion under 10 µs, near the
//! epoch or 100,000 years away, with and without leap rules (research R8).
//!
//! Ignored by default because timing on shared CI machines is noisy. Run it by hand:
//! `cargo test -p uni-calendar --release -- --ignored --nocapture`

mod common;

use std::hint::black_box;
use std::time::{Duration, Instant as Clock};

use common::*;
use uni_calendar::Calendar;
use uni_time::{Instant, Span};

const ROUNDS: i64 = 100_000; // per direction
/// About 100,000 Aster years, in sols.
const FAR_SOLS: i64 = 100_000 * 668;
/// About 100,000 Gregorian years, in days.
const FAR_DAYS: i64 = 36_524_250;

/// Half the instants near `epoch`, half `far` later, a little apart.
fn instants(epoch: Instant, far: Span) -> Vec<Instant> {
    (0..ROUNDS)
        .map(|i| {
            let step = ns(1_234_567_891 * i128::from(i));
            let t = if i % 2 == 0 {
                Ok(epoch)
            } else {
                epoch.checked_add(far)
            };
            t.and_then(|t| t.checked_add(step)).unwrap()
        })
        .collect()
}

/// Converts every instant to a date and every date back, timing each direction.
fn time_both_ways(name: &str, cal: &Calendar, instants: &[Instant]) {
    let dates: Vec<_> = instants
        .iter()
        .map(|&t| cal.instant_to_date(t).unwrap().date)
        .collect();

    let start = Clock::now();
    for &t in instants {
        black_box(cal.instant_to_date(black_box(t)).unwrap());
    }
    let to_date = start.elapsed();

    let start = Clock::now();
    for date in &dates {
        black_box(cal.date_to_instant(black_box(date)).unwrap());
    }
    let to_instant = start.elapsed();

    println!(
        "{name}: {ROUNDS} instant_to_date in {to_date:?}; {ROUNDS} date_to_instant in {to_instant:?}"
    );
    assert!(to_date < Duration::from_secs(1), "too slow: {to_date:?}");
    assert!(
        to_instant < Duration::from_secs(1),
        "too slow: {to_instant:?}"
    );
}

#[test]
#[ignore = "timing check; run by hand in release mode"]
fn a_hundred_thousand_conversions_each_way_take_under_a_second() {
    let cal = Calendar::new(aster(*EPOCH)).unwrap();
    time_both_ways("Aster", &cal, &instants(*EPOCH, sols(FAR_SOLS)));
}

#[test]
#[ignore = "timing check; run by hand in release mode"]
fn with_leap_rules_too() {
    let cal = Calendar::new(aster_with_rule(*EPOCH)).unwrap();
    time_both_ways(
        "Aster with its rule",
        &cal,
        &instants(*EPOCH, sols(FAR_SOLS)),
    );

    let cal = Calendar::new(gregorian_shaped()).unwrap();
    let far = Span::from_nanos(86_400_000_000_000 * i128::from(FAR_DAYS)).unwrap();
    time_both_ways("Gregorian-shaped", &cal, &instants(Instant::ZERO, far));
}
