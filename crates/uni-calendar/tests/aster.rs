//! Hand-worked tests for the spec's Aster calendar (US1 to US3).

mod common;

use common::*;
use uni_calendar::{Calendar, CalendarError};

fn calendar() -> Calendar {
    Calendar::new(aster(*EPOCH)).expect("Aster is valid")
}

mod define {
    use super::*;

    #[test]
    fn aster_is_accepted_with_its_chain() {
        let cal = calendar();
        assert_eq!(cal.name(), "Aster");
        assert_eq!(cal.era_name(), "After Landing");
        assert_eq!(cal.epoch(), *EPOCH);
        assert_eq!(cal.clock(), uni_calendar::Clock::Gct);
        assert_eq!(
            cal.units(),
            ["age", "decade", "year", "month", "sol", "hour", "minute"]
        );
        assert_eq!(cal.era_unit(), "year");
    }

    #[test]
    fn fixed_counts_between_units() {
        let cal = calendar();
        assert_eq!(cal.units_per("year", "sol"), Ok(668));
        assert_eq!(cal.units_per("decade", "sol"), Ok(6_680));
        assert_eq!(cal.units_per("age", "year"), Ok(100));
        assert_eq!(cal.units_per("sol", "minute"), Ok(2_000));
        assert_eq!(cal.units_per("year", "month"), Ok(12));
        assert_eq!(cal.units_per("decade", "month"), Ok(120));
        assert_eq!(cal.units_per("year", "minute"), Ok(668 * 2_000));
        assert_eq!(cal.units_per("sol", "sol"), Ok(1));
    }

    #[test]
    fn counts_that_vary_or_name_no_unit_are_errors() {
        let cal = calendar();
        assert_eq!(
            cal.units_per("month", "sol"),
            Err(CalendarError::NotFixed {
                unit: "month".to_string(),
                of: "sol".to_string()
            })
        );
        assert_eq!(
            cal.units_per("week", "sol"),
            Err(CalendarError::UnknownUnit {
                name: "week".to_string()
            })
        );
        assert_eq!(
            cal.units_per("sol", "week"),
            Err(CalendarError::UnknownUnit {
                name: "week".to_string()
            })
        );
    }

    #[test]
    fn any_sol_length_is_accepted_without_warning() {
        // FR-002a, US1 scenario 7: a calendar needn't match the planet's rotation.
        let mut def = aster(*EPOCH);
        def.fundamental.length = secs(100_000, 1);
        let cal = Calendar::new(def).expect("accepted");
        assert_eq!(cal.units_per("year", "sol"), Ok(668));
    }
}

/// The instant `span` after (or before, if negative) Aster's epoch.
fn at(span: uni_time::Span) -> uni_time::Instant {
    EPOCH.checked_add(span).unwrap()
}

fn date(values: &[i64]) -> uni_calendar::Date {
    uni_calendar::Date {
        values: values.to_vec(),
    }
}

/// Aster with hours numbered from 1 (US1 scenario 6).
fn hours_from_one() -> Calendar {
    let mut def = aster(*EPOCH);
    def.subdivisions[0].first_number = Some(1);
    Calendar::new(def).unwrap()
}

/// A calendar whose only unit is 1 ns long.
fn one_nanosecond() -> Calendar {
    let mut def = aster(*EPOCH);
    def.groupings.clear();
    def.subdivisions.clear();
    def.fundamental.length = secs(1, 1_000_000_000);
    def.era.unit = "sol".to_string();
    Calendar::new(def).unwrap()
}

mod to_date {
    use super::*;
    use uni_calendar::InstantDate;
    use uni_time::{Instant, Ratio};

    fn expect(above: &[i64], values: &[i64], remainder: Ratio) -> InstantDate {
        InstantDate {
            above: above.to_vec(),
            date: date(values),
            remainder,
        }
    }

    #[test]
    fn the_epoch_is_the_first_sol() {
        let cal = calendar();
        assert_eq!(
            cal.instant_to_date(*EPOCH),
            Ok(expect(&[1, 1], &[1, 1, 1, 0, 0], Ratio::ZERO))
        );
        assert_eq!(cal.fixed_from_instant(*EPOCH), Ok(0));
    }

    #[test]
    fn a_boundary_belongs_to_the_unit_that_starts_there() {
        let cal = calendar();
        assert_eq!(
            cal.instant_to_date(at(sols(668))),
            Ok(expect(&[1, 1], &[2, 1, 1, 0, 0], Ratio::ZERO))
        );
        assert_eq!(
            cal.instant_to_date(at(sols(56))),
            Ok(expect(&[1, 1], &[1, 2, 1, 0, 0], Ratio::ZERO))
        );
        assert_eq!(cal.fixed_from_instant(at(sols(56))), Ok(56));
    }

    #[test]
    fn one_nanosecond_before_the_epoch() {
        let cal = calendar();
        let before = at(ns(-1));
        assert_eq!(
            cal.instant_to_date(before),
            Ok(expect(
                &[0, 0],
                &[0, 12, 55, 19, 99],
                secs(44_387_621_999, 1_000_000_000)
            ))
        );
        assert_eq!(cal.fixed_from_instant(before), Ok(-1));
    }

    #[test]
    fn decades_and_ages_count_from_the_epoch() {
        let cal = calendar();
        let year_start = |year: i64| at(sols((year - 1) * 668));
        let above = |year| cal.instant_to_date(year_start(year)).unwrap().above;
        assert_eq!(above(101), [2, 11]);
        assert_eq!(above(-100), [-1, -10]);
        assert_eq!(above(-9), [0, 0]);
        assert_eq!(above(-10), [0, -1]);
        assert_eq!(above(10), [1, 1]);
        assert_eq!(above(11), [1, 2]);
        assert_eq!(
            cal.instant_to_date(year_start(-100)).unwrap().date,
            date(&[-100, 1, 1, 0, 0])
        );
    }

    #[test]
    fn part_way_through_a_minute() {
        // 1,234.5 s into the first sol: minute 27 starts at 27 × 44.387622 = 1,198.465794 s.
        let cal = calendar();
        let t = at(ns(1_234_500_000_000));
        let got = cal.instant_to_date(t).unwrap();
        assert_eq!(
            got,
            expect(&[1, 1], &[1, 1, 1, 0, 27], secs(36_034_206, 1_000_000))
        );
        let minute_start = at(ns(1_198_465_794_000));
        let back = uni_time::Span::from_secs_ratio(got.remainder, uni_time::Rounding::Down);
        assert_eq!(minute_start.checked_add(back.unwrap()), Ok(t));
    }

    #[test]
    fn hours_counted_from_one() {
        assert_eq!(
            hours_from_one().instant_to_date(*EPOCH).unwrap().date,
            date(&[1, 1, 1, 1, 0])
        );
    }

    #[test]
    fn numbers_beyond_i64_are_out_of_range() {
        let cal = one_nanosecond();
        for t in [
            Instant::new(i64::MAX, 999_999_999).unwrap(),
            Instant::new(i64::MIN, 0).unwrap(),
        ] {
            assert_eq!(cal.fixed_from_instant(t), Err(CalendarError::OutOfRange));
            assert_eq!(cal.instant_to_date(t), Err(CalendarError::OutOfRange));
        }
        // Inside the range it still works, a nanosecond at a time.
        assert_eq!(cal.fixed_from_instant(at(ns(-7))), Ok(-7));
        assert_eq!(cal.instant_to_date(at(ns(5))).unwrap().date, date(&[6]));
    }
}
