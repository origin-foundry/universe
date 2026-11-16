//! The Gregorian-shaped test calendar (spec 006, Story 3 scenario 4). Its file name allows
//! Earth's words here (ADR 0009).

use uni_calendar::{
    CalendarDef, Clock, EraDef, FundamentalDef, GroupingDef, MemberDef, MemberUnitDef,
};
use uni_time::{Instant, Ratio};

use super::rule;

/// The Gregorian month lengths of a common year.
pub const GREGORIAN_MONTHS: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// A test calendar shaped like the proleptic Gregorian calendar (spec Story 3, scenario 4): an
/// 86,400 s day, a year of 12 months whose second is 1 day longer by the Gregorian rule, and
/// an era numbering years from `Instant::ZERO`. Not the Gregorian preset.
pub fn gregorian_shaped() -> CalendarDef {
    CalendarDef {
        name: "Gregorian-shaped".to_string(),
        clock: Clock::Gct,
        groupings: vec![GroupingDef {
            name: "year".to_string(),
            member_unit: Some(MemberUnitDef {
                name: "month".to_string(),
                first_number: None,
            }),
            members: GREGORIAN_MONTHS
                .iter()
                .map(|&count| MemberDef { name: None, count })
                .collect(),
            first_number: None,
            rules: vec![rule(
                "year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)",
                "2",
                1,
            )],
        }],
        fundamental: FundamentalDef {
            name: "day".to_string(),
            length: Ratio::from_int(86_400).unwrap(),
            first_number: None,
        },
        subdivisions: vec![],
        era: EraDef {
            name: "Common Era".to_string(),
            unit: "year".to_string(),
            epoch: Instant::ZERO,
        },
    }
}
