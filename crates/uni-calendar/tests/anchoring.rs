//! `Calendar::epoch_for` (spec 008, research R1): the epoch that puts a date at an instant.

mod common;

use common::*;
use uni_calendar::{
    Calendar, CalendarDef, CalendarError, Clock, Date, EraDef, FundamentalDef, GroupingDef,
    MemberDef,
};
use uni_time::{Instant, Ratio};

fn date(values: &[i64]) -> Date {
    Date {
        values: values.to_vec(),
    }
}

fn one_member(name: &str, count: i64) -> GroupingDef {
    GroupingDef {
        name: name.to_string(),
        member_unit: None,
        members: vec![MemberDef { name: None, count }],
        first_number: None,
        rules: vec![],
    }
}

/// The spec's Tallow: a cycle of 52 rounds, a round of 7 watches, a watch of 30,000 s; the
/// era numbers cycles.
fn tallow(epoch: Instant) -> CalendarDef {
    CalendarDef {
        name: "Tallow".to_string(),
        clock: Clock::Gct,
        groupings: vec![one_member("cycle", 52), one_member("round", 7)],
        fundamental: FundamentalDef {
            name: "watch".to_string(),
            length: Ratio::from_int(30_000).unwrap(),
            first_number: None,
        },
        subdivisions: vec![],
        era: EraDef {
            name: "Count".to_string(),
            unit: "cycle".to_string(),
            epoch,
        },
    }
}

/// A calendar of one unit, `355/113` s long, numbered by the era.
fn awkward(epoch: Instant) -> CalendarDef {
    CalendarDef {
        name: "Awkward".to_string(),
        clock: Clock::Gct,
        groupings: vec![],
        fundamental: FundamentalDef {
            name: "beat".to_string(),
            length: Ratio::new(355, 113).unwrap(),
            first_number: None,
        },
        subdivisions: vec![],
        era: EraDef {
            name: "Count".to_string(),
            unit: "beat".to_string(),
            epoch,
        },
    }
}

/// `def` rebuilt with `epoch`.
fn moved(mut def: CalendarDef, epoch: Instant) -> Calendar {
    def.era.epoch = epoch;
    Calendar::new(def).unwrap()
}

#[test]
fn tallow_starts_at_an_aster_date() {
    let aster = Calendar::new(aster(*EPOCH)).unwrap();
    let at = aster.date_to_instant(&date(&[412, 3, 9])).unwrap().instant;
    let cal = Calendar::new(tallow(Instant::ZERO)).unwrap();
    let epoch = cal.epoch_for(&date(&[1, 1, 1]), at).unwrap();
    // [1, 1, 1] is the first watch, so the epoch is the moment itself.
    assert_eq!(epoch, at);
    let placed = moved(tallow(Instant::ZERO), epoch);
    assert_eq!(
        placed.date_to_instant(&date(&[1, 1, 1])).unwrap().instant,
        at
    );
    // And back: that moment is orbit 412, span 3, sol 9 at its very start in Aster.
    let back = aster.instant_to_date(at).unwrap();
    assert_eq!(back.date.values, [412, 3, 9, 0, 0]);
    assert_eq!(back.remainder, Ratio::ZERO);
}

#[test]
fn a_later_date_moves_the_epoch_back() {
    let cal = Calendar::new(tallow(*EPOCH)).unwrap();
    // Cycle 2, round 3, watch 4: 52 × 7 + 2 × 7 + 3 = 381 watches after the epoch.
    let epoch = cal.epoch_for(&date(&[2, 3, 4]), Instant::ZERO).unwrap();
    assert_eq!(epoch, Instant::new(-381 * 30_000, 0).unwrap());
    let placed = moved(tallow(*EPOCH), epoch);
    assert_eq!(
        placed.date_to_instant(&date(&[2, 3, 4])).unwrap().instant,
        Instant::ZERO
    );
}

#[test]
fn dates_before_the_epoch() {
    let at = Instant::new(5_000_000, 0).unwrap();
    for values in [vec![0], vec![-3, 2]] {
        let cal = Calendar::new(tallow(*EPOCH)).unwrap();
        let epoch = cal.epoch_for(&date(&values), at).unwrap();
        assert!(epoch > at, "{values:?} is before the epoch");
        let placed = moved(tallow(*EPOCH), epoch);
        assert_eq!(placed.date_to_instant(&date(&values)).unwrap().instant, at);
    }
}

#[test]
fn a_start_between_whole_nanoseconds_still_lands_exactly() {
    let cal = Calendar::new(awkward(*EPOCH)).unwrap();
    let at = Instant::new(1_000, 1).unwrap();
    let beat = date(&[8]); // 7 × 355/113 s after the epoch: not a whole nanosecond
    assert!(
        cal.date_to_instant(&beat).unwrap().rounded_by.is_some(),
        "the start is rounded"
    );
    let placed = moved(awkward(*EPOCH), cal.epoch_for(&beat, at).unwrap());
    let start = placed.date_to_instant(&beat).unwrap();
    assert_eq!(start.instant, at);
    assert!(start.rounded_by.is_some());
}

#[test]
fn an_invalid_date_is_the_engines_error() {
    let cal = Calendar::new(aster(*EPOCH)).unwrap();
    assert_eq!(
        cal.epoch_for(&date(&[412, 13]), Instant::ZERO)
            .unwrap_err()
            .to_string(),
        "span 13 is outside 1 to 12"
    );
    assert!(matches!(
        cal.epoch_for(&date(&[412, 13]), Instant::ZERO),
        Err(CalendarError::InvalidValue { .. })
    ));
    assert_eq!(
        cal.epoch_for(&date(&[]), Instant::ZERO),
        Err(CalendarError::EmptyDate)
    );
}

#[test]
fn an_epoch_beyond_the_timeline_is_out_of_range() {
    let cal = Calendar::new(tallow(*EPOCH)).unwrap();
    let last = Instant::new(i64::MAX, 999_999_999).unwrap();
    let first = Instant::new(i64::MIN, 0).unwrap();
    // Before the epoch, near the end: the epoch would be after the end.
    assert_eq!(
        cal.epoch_for(&date(&[-3]), last),
        Err(CalendarError::OutOfRange)
    );
    // After the epoch, near the start: the epoch would be before the start.
    assert_eq!(
        cal.epoch_for(&date(&[5]), first),
        Err(CalendarError::OutOfRange)
    );
    // The first date at the very end is fine: the epoch is that instant.
    assert_eq!(cal.epoch_for(&date(&[1]), last), Ok(last));
}
