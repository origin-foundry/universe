//! Hand-worked tests for Tessel (research R7): unnamed phases, turns counted from 0, and watch
//! boundaries that fall between nanoseconds.

mod common;

use common::*;
use uni_calendar::{Calendar, CalendarError, Date, DateStart, InstantDate};
use uni_time::Ratio;

fn calendar() -> Calendar {
    Calendar::new(tessel(*EPOCH)).expect("Tessel is valid")
}

fn date(values: &[i64]) -> Date {
    Date {
        values: values.to_vec(),
    }
}

/// `n` turns of 100,000 s.
fn turns(n: i64) -> uni_time::Span {
    ns(100_000_000_000_000 * i128::from(n))
}

#[test]
fn tessel_is_accepted() {
    let cal = calendar();
    assert_eq!(cal.units(), ["cycle", "phase", "turn", "watch"]);
    assert_eq!(cal.era_unit(), "cycle");
    assert_eq!(cal.units_per("cycle", "turn"), Ok(40));
    assert_eq!(cal.units_per("cycle", "phase"), Ok(3));
    assert_eq!(cal.units_per("turn", "watch"), Ok(7));
}

#[test]
fn a_watch_starting_between_nanoseconds_is_rounded_up_and_says_so() {
    // Watch 1 starts at T + 100,000/7 s = T + 14,285,714,285,714 2/7 ns.
    let cal = calendar();
    let start = cal.date_to_instant(&date(&[1, 1, 0, 1])).unwrap();
    assert_eq!(
        start,
        DateStart {
            instant: EPOCH.checked_add(ns(14_285_714_285_715)).unwrap(),
            rounded_by: Some(secs(5, 7_000_000_000)),
        }
    );
    // The rounded instant is still inside watch 1, 5/7 ns after its exact start.
    assert_eq!(
        cal.instant_to_date(start.instant),
        Ok(InstantDate {
            above: vec![],
            date: date(&[1, 1, 0, 1]),
            remainder: secs(5, 7_000_000_000),
        })
    );
}

#[test]
fn whole_nanosecond_starts_are_not_rounded() {
    let cal = calendar();
    assert_eq!(
        cal.date_to_instant(&date(&[1, 1, 0, 0])),
        Ok(DateStart {
            instant: *EPOCH,
            rounded_by: None
        })
    );
    assert_eq!(
        cal.instant_to_date(*EPOCH).map(|d| d.date),
        Ok(date(&[1, 1, 0, 0]))
    );
}

#[test]
fn phase_two_starts_at_turn_thirteen() {
    let cal = calendar();
    let start = EPOCH.checked_add(turns(13)).unwrap();
    assert_eq!(
        cal.date_to_instant(&date(&[1, 2, 0])),
        Ok(DateStart {
            instant: start,
            rounded_by: None
        })
    );
    assert_eq!(cal.fixed_from_instant(start), Ok(13));
}

#[test]
fn turns_run_from_zero_within_their_phase() {
    assert_eq!(
        calendar().date_to_instant(&date(&[1, 1, 13])),
        Err(CalendarError::InvalidValue {
            unit: "turn".to_string(),
            value: 13,
            first: 0,
            last: 12,
            within: Some("1".to_string()),
            year: None,
        })
    );
    // The third phase has 14 turns.
    assert!(calendar().date_to_instant(&date(&[1, 3, 13])).is_ok());
}

#[test]
fn the_last_watch_before_the_epoch() {
    // Starts at T − 100,000/7 s = T − 14,285,714,285,714 2/7 ns: rounded up by 2/7 ns.
    let cal = calendar();
    let start = cal.date_to_instant(&date(&[0, 3, 13, 6])).unwrap();
    assert_eq!(
        start,
        DateStart {
            instant: EPOCH.checked_add(ns(-14_285_714_285_714)).unwrap(),
            rounded_by: Some(secs(2, 7_000_000_000)),
        }
    );
    let back = cal
        .instant_to_date(EPOCH.checked_add(ns(-1)).unwrap())
        .unwrap();
    assert_eq!(back.date, date(&[0, 3, 13, 6]));
    // 1 ns before the epoch is 1 ns before the end of the watch: 100,000/7 s − 1 ns in.
    assert_eq!(
        back.remainder,
        Ratio::new(100_000_000_000_000 - 7, 7_000_000_000).unwrap()
    );
}
