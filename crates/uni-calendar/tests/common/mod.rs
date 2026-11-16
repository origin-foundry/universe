//! The test calendars shared by the integration tests: Aster (spec.md) and Tessel (research R7),
//! with invented unit names (ADR 0009), and the Gregorian-shaped calendar in `gregorian.rs`.

#![allow(dead_code)] // each test file uses only some of these

use std::sync::LazyLock;

use uni_calendar::{
    AdjustmentRuleDef, CalendarDef, Clock, EraDef, FundamentalDef, GroupingDef, MemberDef,
    MemberUnitDef, SubdivisionDef,
};
use uni_time::{Instant, Ratio, Span};

mod gregorian;
#[allow(unused_imports)] // only some test files use it
pub use gregorian::*;

/// An arbitrary epoch away from the timeline's zero, so nothing passes by accident.
/// (`Instant::new` isn't `const`, so this is a lazily built static: write `*EPOCH`.)
pub static EPOCH: LazyLock<Instant> =
    LazyLock::new(|| Instant::new(1_234_567_890, 123_456_789).unwrap());

/// Aster's sol, 88,775.244 s, in nanoseconds.
pub const SOL_NS: i128 = 88_775_244_000_000;

pub const SPANS: [(&str, i64); 12] = [
    ("Prime", 56),
    ("Second", 56),
    ("Third", 55),
    ("Fourth", 56),
    ("Fifth", 56),
    ("Sixth", 55),
    ("Seventh", 56),
    ("Eighth", 56),
    ("Ninth", 55),
    ("Tenth", 56),
    ("Eleventh", 56),
    ("Twelfth", 55),
];

fn one_member(name: &str, count: i64) -> GroupingDef {
    GroupingDef {
        name: name.to_string(),
        member_unit: None,
        members: vec![MemberDef { name: None, count }],
        first_number: None,
        rules: vec![],
    }
}

fn subdivision(name: &str, parts: i64) -> SubdivisionDef {
    SubdivisionDef {
        name: name.to_string(),
        parts,
        first_number: None,
    }
}

/// The spec's Aster: age (10 cohorts), cohort (10 orbits), orbit (12 spans in sols), sol of
/// 88,775.244 s, bell (20 parts), pulse (100 parts); the era "After Landing" numbers orbits.
pub fn aster(epoch: Instant) -> CalendarDef {
    CalendarDef {
        name: "Aster".to_string(),
        clock: Clock::Gct,
        groupings: vec![
            one_member("age", 10),
            one_member("cohort", 10),
            GroupingDef {
                name: "orbit".to_string(),
                member_unit: Some(MemberUnitDef {
                    name: "span".to_string(),
                    first_number: None,
                }),
                members: SPANS
                    .iter()
                    .map(|&(name, count)| MemberDef {
                        name: Some(name.to_string()),
                        count,
                    })
                    .collect(),
                first_number: None,
                rules: vec![],
            },
        ],
        fundamental: FundamentalDef {
            name: "sol".to_string(),
            length: Ratio::new(88_775_244, 1000).unwrap(),
            first_number: None,
        },
        subdivisions: vec![subdivision("bell", 20), subdivision("pulse", 100)],
        era: EraDef {
            name: "After Landing".to_string(),
            unit: "orbit".to_string(),
            epoch,
        },
    }
}

/// Tessel (research R7): a cycle of three unnamed phases of 13, 13 and 14 turns; a turn of
/// exactly 100,000 s counted from 0, split into 7 watches; the era numbers cycles.
pub fn tessel(epoch: Instant) -> CalendarDef {
    CalendarDef {
        name: "Tessel".to_string(),
        clock: Clock::Gct,
        groupings: vec![GroupingDef {
            name: "cycle".to_string(),
            member_unit: Some(MemberUnitDef {
                name: "phase".to_string(),
                first_number: None,
            }),
            members: [13, 13, 14]
                .iter()
                .map(|&count| MemberDef { name: None, count })
                .collect(),
            first_number: None,
            rules: vec![],
        }],
        fundamental: FundamentalDef {
            name: "turn".to_string(),
            length: Ratio::from_int(100_000).unwrap(),
            first_number: Some(0),
        },
        subdivisions: vec![subdivision("watch", 7)],
        era: EraDef {
            name: "Weave".to_string(),
            unit: "cycle".to_string(),
            epoch,
        },
    }
}

/// `n` Aster sols.
pub fn sols(n: i64) -> Span {
    Span::from_nanos(SOL_NS * i128::from(n)).unwrap()
}

/// `n` nanoseconds.
pub fn ns(n: i128) -> Span {
    Span::from_nanos(n).unwrap()
}

/// An exact number of seconds, `num / den`.
pub fn secs(num: i128, den: i128) -> Ratio {
    Ratio::new(num, den).unwrap()
}

/// Aster with the spec's adjustment rule: Twelfth is 1 sol longer when
/// `orbit % 5 == 0 && (orbit % 100 != 0 || orbit % 500 == 0)` (rule period 500).
pub fn aster_with_rule(epoch: Instant) -> CalendarDef {
    let mut def = aster(epoch);
    def.groupings[2].rules = vec![rule(
        "orbit % 5 == 0 && (orbit % 100 != 0 || orbit % 500 == 0)",
        "Twelfth",
        1,
    )];
    def
}

/// Tessel with a shortening rule on an unnamed member (research R8): phase 3 is 1 turn shorter
/// (13 instead of 14) when `cycle % 3 == 0`.
pub fn tessel_with_rule(epoch: Instant) -> CalendarDef {
    let mut def = tessel(epoch);
    def.groupings[0].rules = vec![rule("cycle % 3 == 0", "3", -1)];
    def
}

/// An adjustment rule.
pub fn rule(condition: &str, member: &str, change: i64) -> AdjustmentRuleDef {
    AdjustmentRuleDef {
        condition: condition.to_string(),
        member: member.to_string(),
        change,
    }
}
