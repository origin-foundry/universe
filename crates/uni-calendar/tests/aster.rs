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

mod to_instant {
    use super::*;
    use uni_calendar::DateStart;

    fn exact(instant: uni_time::Instant) -> Result<DateStart, CalendarError> {
        Ok(DateStart {
            instant,
            rounded_by: None,
        })
    }

    fn invalid(
        unit: &str,
        value: i64,
        first: i64,
        last: i64,
        within: Option<&str>,
    ) -> CalendarError {
        CalendarError::InvalidValue {
            unit: unit.to_string(),
            value,
            first,
            last,
            within: within.map(str::to_string),
        }
    }

    #[test]
    fn the_first_sol_starts_at_the_epoch() {
        let cal = calendar();
        assert_eq!(cal.date_to_instant(&date(&[1, 1, 1])), exact(*EPOCH));
        assert_eq!(cal.start_of_fixed(0), exact(*EPOCH));
        assert_eq!(cal.start_of_fixed(-1), exact(at(sols(-1))));
        assert_eq!(cal.start_of_fixed(668), exact(at(sols(668))));
    }

    #[test]
    fn a_sol_past_the_end_of_its_month() {
        assert_eq!(
            calendar().date_to_instant(&date(&[1, 3, 56])),
            Err(invalid("sol", 56, 1, 55, Some("Third")))
        );
    }

    #[test]
    fn values_outside_their_range_are_never_wrapped() {
        let cal = calendar();
        let cases = [
            (vec![1, 13], invalid("month", 13, 1, 12, None)),
            (vec![1, 1, 0], invalid("sol", 0, 1, 56, Some("Prime"))),
            (vec![1, 1, 1, 20], invalid("hour", 20, 0, 19, None)),
            (vec![1, 1, 1, 0, -1], invalid("minute", -1, 0, 99, None)),
        ];
        for (values, error) in cases {
            assert_eq!(
                cal.date_to_instant(&date(&values)),
                Err(error),
                "{values:?}"
            );
        }
        assert_eq!(
            hours_from_one().date_to_instant(&date(&[1, 1, 1, 0])),
            Err(invalid("hour", 0, 1, 20, None))
        );
    }

    #[test]
    fn missing_smaller_units_take_their_first_value() {
        let cal = calendar();
        assert_eq!(cal.date_to_instant(&date(&[1, 2])), exact(at(sols(56))));
        assert_eq!(cal.date_to_instant(&date(&[1])), exact(*EPOCH));
        assert_eq!(cal.date_to_instant(&date(&[2])), exact(at(sols(668))));
    }

    #[test]
    fn empty_and_overlong_dates() {
        let cal = calendar();
        assert_eq!(
            cal.date_to_instant(&date(&[])),
            Err(CalendarError::EmptyDate)
        );
        assert_eq!(
            cal.date_to_instant(&date(&[1, 1, 1, 0, 0, 0])),
            Err(CalendarError::TooManyValues { given: 6, max: 5 })
        );
    }

    #[test]
    fn before_the_epoch_and_out_of_range() {
        let cal = calendar();
        assert_eq!(
            cal.date_to_instant(&date(&[0, 12, 55])),
            exact(at(sols(-1)))
        );
        assert_eq!(
            cal.date_to_instant(&date(&[i64::MAX])),
            Err(CalendarError::OutOfRange)
        );
        assert_eq!(
            cal.date_to_instant(&date(&[i64::MIN])),
            Err(CalendarError::OutOfRange)
        );
        assert_eq!(cal.start_of_fixed(i64::MAX), Err(CalendarError::OutOfRange));
        let tiny = one_nanosecond();
        assert_eq!(tiny.date_to_instant(&date(&[1])), exact(*EPOCH));
        assert_eq!(tiny.start_of_fixed(-7), exact(at(ns(-7))));
    }

    #[test]
    fn every_hand_worked_date_converts_back() {
        let cal = calendar();
        let instants = [
            *EPOCH,
            at(sols(668)),
            at(sols(56)),
            at(ns(-1)),
            at(sols(100 * 668)),
            at(sols(-101 * 668)),
            at(sols(-10 * 668)),
            at(sols(-11 * 668)),
            at(ns(1_234_500_000_000)),
        ];
        for t in instants {
            let got = cal.instant_to_date(t).unwrap();
            let start = cal.date_to_instant(&got.date).unwrap();
            assert_eq!(start.rounded_by, None);
            let back = uni_time::Span::from_secs_ratio(got.remainder, uni_time::Rounding::Down);
            assert_eq!(start.instant.checked_add(back.unwrap()), Ok(t), "{t:?}");
        }
    }
}
