//! Performance check for SC-005: each operation well under a microsecond.
//!
//! Ignored by default because timing on shared CI machines is noisy. Run it by hand:
//! `cargo test -p uni-time --release -- --ignored --nocapture`

use std::hint::black_box;
use std::time::{Duration, Instant as Clock};

use uni_time::{Instant, Ratio, Rounding, Span};

const ROUNDS: u32 = 200_000; // five operations each: a million in total

#[test]
#[ignore = "timing check; run by hand in release mode"]
fn a_million_operations_take_under_a_second() {
    let mut t = Instant::new(-3_155_695_200_000, 123).unwrap();
    let step = Span::from_nanos(1_234_567_891).unwrap();
    let mut q = Ratio::new(146_097, 400).unwrap();
    let small = Ratio::new(1, 7).unwrap();

    let start = Clock::now();
    for i in 0..ROUNDS {
        t = black_box(t.checked_add(step).unwrap());
        let gap = black_box(t.since(Instant::ZERO));
        q = black_box(q.checked_add(small).unwrap());
        let _ = black_box(q.checked_mul(small).unwrap());
        let _ = black_box(Span::from_secs_ratio(Ratio::new(i128::from(i), 3).unwrap(), Rounding::NearestEven));
        black_box(gap);
    }
    let elapsed = start.elapsed();

    println!("{} operations in {elapsed:?}", ROUNDS * 5);
    assert!(elapsed < Duration::from_secs(1), "too slow: {elapsed:?}");
}
