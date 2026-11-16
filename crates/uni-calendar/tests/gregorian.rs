//! Story 3 scenario 4 (SC-005): every day of years 1 to 4000 of the Gregorian-shaped test
//! calendar has the day number Python's `datetime` gives it. The fixture and the script that
//! wrote it are in `tests/data/`; Python shares no code with the engine (constitution VI).

mod common;

use common::gregorian_shaped;
use uni_calendar::{Calendar, CalendarError, Date};

const FIXTURE: &str = include_str!("data/gregorian_days.txt");

/// One fixture line: the year, the ordinal of its 1 January, and its month lengths.
struct Year {
    year: i64,
    jan1: i64,
    months: Vec<i64>,
}

fn fixture() -> Vec<Year> {
    FIXTURE
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let numbers: Vec<i64> = line
                .split_whitespace()
                .map(|n| n.parse().unwrap())
                .collect();
            assert_eq!(numbers.len(), 14, "bad fixture line: {line}");
            Year {
                year: numbers[0],
                jan1: numbers[1],
                months: numbers[2..].to_vec(),
            }
        })
        .collect()
}

fn date(values: &[i64]) -> Date {
    Date {
        values: values.to_vec(),
    }
}

#[test]
fn every_day_of_four_thousand_years_matches_python() {
    let cal = Calendar::new(gregorian_shaped()).unwrap();
    assert_eq!(cal.rule_period(), 400);
    let years = fixture();
    assert_eq!(years.len(), 4000);
    assert_eq!(
        (years[0].year, years[3999].year),
        (1, 4000),
        "the fixture covers years 1 to 4000"
    );
    let mut days = 0u64;
    for y in &years {
        assert_eq!(
            cal.era_instance(y.year).unwrap().members,
            y.months,
            "month lengths of {}",
            y.year
        );
        // Fixed number 0 is 1 January of year 1, ordinal 1.
        let mut fixed = y.jan1 - 1;
        for (m, &length) in (1..).zip(&y.months) {
            for d in 1..=length {
                let start = cal.date_to_instant(&date(&[y.year, m, d])).unwrap();
                assert_eq!(start.rounded_by, None);
                assert_eq!(
                    cal.fixed_from_instant(start.instant),
                    Ok(fixed),
                    "{}-{m}-{d}",
                    y.year
                );
                assert_eq!(
                    cal.instant_to_date(start.instant).unwrap().date,
                    date(&[y.year, m, d]),
                    "{}-{m}-{d}",
                    y.year
                );
                fixed += 1;
                days += 1;
            }
            // The day after the month's last doesn't exist.
            assert!(
                matches!(
                    cal.date_to_instant(&date(&[y.year, m, length + 1])),
                    Err(CalendarError::InvalidValue { .. })
                ),
                "{}-{m}-{}",
                y.year,
                length + 1
            );
        }
    }
    // 4000 years of 365 days and 970 leap days (ten 400-year cycles of 97).
    assert_eq!(days, 4000 * 365 + 970);
}
