//! Hand-worked tests for adjustment rules (spec 006): defining calendars with rules (`define_`,
//! Story 1) and converting dates in them (`convert_`, Story 2), on Aster and Tessel.

mod common;

use common::*;
use uni_calendar::{
    AdjustmentRuleDef, Calendar, CalendarDef, CalendarError, Date, DefinitionError, EraInstance,
    EraNumber,
};
use uni_expr::{CheckError, EvalError, PeriodError, Pos};

fn s(text: &str) -> String {
    text.to_string()
}

fn at(column: u32) -> Pos {
    Pos { line: 1, column }
}

/// Aster with `rules` on its orbit.
fn aster_rules(rules: Vec<AdjustmentRuleDef>) -> CalendarDef {
    let mut def = aster(*EPOCH);
    def.groupings[2].rules = rules;
    def
}

fn refused(def: CalendarDef) -> DefinitionError {
    Calendar::new(def).expect_err("should be refused")
}

fn not_fixed(unit: &str, of: &str) -> Result<i64, CalendarError> {
    Err(CalendarError::NotFixed {
        unit: s(unit),
        of: s(of),
    })
}

// -------------------------------------------------------------------------------------------
// Story 1: define a calendar with adjustment rules

/// Scenario 1.
#[test]
fn define_aster_with_its_rule_is_accepted() {
    let cal = Calendar::new(aster_with_rule(*EPOCH)).unwrap();
    assert_eq!(cal.rule_period(), 500);
    assert_eq!(cal.units_per("orbit", "sol"), not_fixed("orbit", "sol"));
    assert_eq!(cal.units_per("cohort", "sol"), not_fixed("cohort", "sol"));
    assert_eq!(cal.units_per("orbit", "span"), Ok(12));
    assert_eq!(cal.units_per("cohort", "orbit"), Ok(10));
    assert_eq!(cal.units_per("age", "span"), Ok(1_200));
    // Inside a sol nothing changes.
    assert_eq!(cal.units_per("sol", "pulse"), Ok(2_000));

    let plain = Calendar::new(aster(*EPOCH)).unwrap();
    assert_eq!(plain.rule_period(), 1);
    assert_eq!(plain.units_per("orbit", "sol"), Ok(668));
}

/// Scenario 2.
#[test]
fn define_conditions_the_expression_language_refuses() {
    match refused(aster_rules(vec![rule("orbit % 4", "Twelfth", 1)])) {
        DefinitionError::RuleCondition {
            unit,
            rule: 1,
            error: CheckError::TypeMismatch {
                context: "result", ..
            },
        } => assert_eq!(unit, "orbit"),
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(
        refused(aster_rules(vec![rule("span == 2", "Twelfth", 1)])),
        DefinitionError::RuleCondition {
            unit: s("orbit"),
            rule: 1,
            error: CheckError::UnknownVariable {
                name: s("span"),
                at: at(1)
            },
        }
    );
}

/// Scenario 3.
#[test]
fn define_unknown_members_and_zero_changes() {
    assert_eq!(
        refused(aster_rules(vec![rule("orbit % 4 == 0", "Thirteenth", 1)])),
        DefinitionError::RuleUnknownMember {
            unit: s("orbit"),
            rule: 1,
            member: s("Thirteenth"),
        }
    );
    assert_eq!(
        refused(aster_rules(vec![rule("orbit % 4 == 0", "Twelfth", 0)])),
        DefinitionError::RuleZeroChange {
            unit: s("orbit"),
            rule: 1
        }
    );
    // A second rule's fault names rule 2.
    assert_eq!(
        refused(aster_rules(vec![
            rule("orbit % 4 == 0", "Twelfth", 1),
            rule("orbit % 7 == 0", "Third", 0),
        ])),
        DefinitionError::RuleZeroChange {
            unit: s("orbit"),
            rule: 2
        }
    );
}

/// Scenario 4.
#[test]
fn define_members_shortened_to_nothing() {
    assert_eq!(
        refused(aster_rules(vec![rule("orbit % 2 == 0", "Third", -55)])),
        DefinitionError::MemberTooShort {
            unit: s("orbit"),
            member: s("Third"),
            era_number: 2,
            length: 0,
        }
    );
    // Changes add up: 55 − 30 − 30.
    assert_eq!(
        refused(aster_rules(vec![
            rule("orbit % 2 == 0", "Third", -30),
            rule("orbit % 2 == 0", "Third", -30),
        ])),
        DefinitionError::MemberTooShort {
            unit: s("orbit"),
            member: s("Third"),
            era_number: 2,
            length: -5,
        }
    );
}

/// Scenario 5.
#[test]
fn define_conditions_that_fail_for_some_orbit() {
    // Orbit 7 is the first where `orbit % 7` is 0; `/` is at column 5.
    assert_eq!(
        refused(aster_rules(vec![rule(
            "100 / (orbit % 7) == 3",
            "Twelfth",
            1
        )])),
        DefinitionError::RuleFails {
            unit: s("orbit"),
            rule: 1,
            era_number: 7,
            error: EvalError::DivisionByZero { at: at(5) },
        }
    );
    // 2 × 2⁶² overflows: orbit 2.
    match refused(aster_rules(vec![rule(
        "orbit * 4611686018427387904 % 3 == 0",
        "Twelfth",
        1,
    )])) {
        DefinitionError::RuleFails {
            unit,
            rule: 1,
            era_number: 2,
            error: EvalError::OutOfRange { .. },
        } => assert_eq!(unit, "orbit"),
        other => panic!("unexpected {other:?}"),
    }
}

/// Scenario 6.
#[test]
fn define_rules_only_on_the_era_unit() {
    let mut def = aster(*EPOCH);
    def.groupings[1].rules = vec![rule("cohort % 2 == 0", "1", 1)];
    assert_eq!(
        refused(def),
        DefinitionError::RulesNotOnEraUnit { unit: s("cohort") }
    );
    // An era that numbers sols: the orbit isn't the era's unit.
    let mut def = aster_with_rule(*EPOCH);
    def.era.unit = s("sol");
    assert_eq!(
        refused(def),
        DefinitionError::RulesNotOnEraUnit { unit: s("orbit") }
    );
}

/// Scenario 7.
#[test]
fn define_rules_that_dont_repeat_within_the_limit() {
    assert_eq!(
        refused(aster_rules(vec![rule("orbit > 100", "Twelfth", 1)])),
        DefinitionError::RulePeriod {
            unit: s("orbit"),
            rule: 1,
            error: PeriodError::NotRepeating { at: at(7) },
        }
    );
    assert_eq!(
        refused(aster_rules(vec![rule("orbit % 100003 == 0", "Twelfth", 1)])),
        DefinitionError::RulePeriod {
            unit: s("orbit"),
            rule: 1,
            error: PeriodError::OverLimit { limit: 100_000 },
        }
    );
    // Each is fine, but together they repeat every 97 × 89 × 83 = 716,539 orbits.
    assert_eq!(
        refused(aster_rules(vec![
            rule("orbit % 97 == 0", "Twelfth", 1),
            rule("orbit % 89 == 0", "Twelfth", 1),
            rule("orbit % 83 == 0", "Twelfth", 1),
        ])),
        DefinitionError::RulesPeriodOverLimit {
            unit: s("orbit"),
            limit: 100_000
        }
    );
}

/// An era's unit whose name can't be a variable name (FR-005).
#[test]
fn define_an_era_unit_named_like_a_keyword() {
    let mut def = aster(*EPOCH);
    def.groupings[2].name = s("if");
    def.era.unit = s("if");
    assert!(Calendar::new(def.clone()).is_ok(), "fine without rules");
    def.groupings[2].rules = vec![rule("true", "Twelfth", 1)];
    match refused(def) {
        DefinitionError::RuleCondition {
            unit,
            rule: 1,
            error: CheckError::InvalidVariable { name, .. },
        } => {
            assert_eq!(unit, "if");
            assert_eq!(name, "if");
        }
        other => panic!("unexpected {other:?}"),
    }
}

/// A second calendar: a shortening rule on an unnamed member, reading `cycle`.
#[test]
fn define_tessel_with_its_rule_is_accepted() {
    let cal = Calendar::new(tessel_with_rule(*EPOCH)).unwrap();
    assert_eq!(cal.rule_period(), 3);
    assert_eq!(cal.units_per("cycle", "phase"), Ok(3));
    assert_eq!(cal.units_per("cycle", "turn"), not_fixed("cycle", "turn"));
    // Members without names are given by number from 1.
    let mut def = tessel(*EPOCH);
    def.groupings[0].rules = vec![rule("cycle % 3 == 0", "4", -1)];
    assert_eq!(
        refused(def),
        DefinitionError::RuleUnknownMember {
            unit: s("cycle"),
            rule: 1,
            member: s("4"),
        }
    );
}

// -------------------------------------------------------------------------------------------
// Story 2: convert dates in a calendar with adjustment rules

fn aster_cal() -> Calendar {
    Calendar::new(aster_with_rule(*EPOCH)).unwrap()
}

fn date(values: &[i64]) -> Date {
    Date {
        values: values.to_vec(),
    }
}

/// The fixed number a date starts at.
fn fixed(cal: &Calendar, values: &[i64]) -> i64 {
    let start = cal.date_to_instant(&date(values)).unwrap();
    cal.fixed_from_instant(start.instant).unwrap()
}

fn era(unit: &str, number: i64) -> Option<EraNumber> {
    Some(EraNumber {
        unit: s(unit),
        number,
    })
}

/// Scenario 1: orbits 1 to 4 are common (4 × 668); Twelfth starts 613 sols into orbit 5 and sol
/// 56 is 55 sols further: 2,672 + 613 + 55 = 3,340.
#[test]
fn convert_the_extra_sol_of_orbit_five() {
    let cal = aster_cal();
    let start = cal.date_to_instant(&date(&[5, 12, 56])).unwrap();
    assert_eq!(start.instant, EPOCH.checked_add(sols(3_340)).unwrap());
    assert_eq!(cal.fixed_from_instant(start.instant), Ok(3_340));
    let back = cal.instant_to_date(start.instant).unwrap();
    assert_eq!(back.date, date(&[5, 12, 56, 0, 0]));
    assert_eq!(back.above, [1, 1]);
}

/// Scenario 2.
#[test]
fn convert_refuses_the_extra_sol_in_a_common_orbit() {
    assert_eq!(
        aster_cal().date_to_instant(&date(&[1, 12, 56])),
        Err(CalendarError::InvalidValue {
            unit: s("sol"),
            value: 56,
            first: 1,
            last: 55,
            within: Some(s("Twelfth")),
            era_number: era("orbit", 1),
        })
    );
}

/// Scenarios 3 and 4, and orbits before the epoch.
#[test]
fn convert_orbit_starts() {
    let cal = aster_cal();
    // Five orbits, one of them (5) a long orbit: 5 × 668 + 1.
    assert_eq!(fixed(&cal, &[6]), 3_341);
    // One full repetition: 500 × 668 + 96 long orbits.
    assert_eq!(fixed(&cal, &[501]), 334_096);
    // Orbit 0 is a long orbit (0 is a multiple of 500): it starts 669 sols before the epoch.
    assert_eq!(fixed(&cal, &[0]), -669);
    // Orbits −4 to −1 are common (−4 % 5 = 1, …), orbit 0 long: 4 × 668 + 669 = 3,341.
    assert_eq!(fixed(&cal, &[-4]), -3_341);
    // Orbit −5 is long (−5 % 100 = 95): it starts another 669 sols earlier.
    assert_eq!(fixed(&cal, &[-5]), -4_010);
}

/// Scenario 5: the last pulse of orbit 0, a long orbit, so Twelfth sol 56.
#[test]
fn convert_one_nanosecond_before_the_epoch() {
    let got = aster_cal()
        .instant_to_date(EPOCH.checked_add(ns(-1)).unwrap())
        .unwrap();
    assert_eq!(got.above, [0, 0]);
    assert_eq!(got.date, date(&[0, 12, 56, 19, 99]));
    // A pulse is 88,775.244 s / 2,000 = 44.387622 s.
    assert_eq!(got.remainder, secs(44_387_621_999, 1_000_000_000));
}

/// Scenario 6.
#[test]
fn convert_age_orbits() {
    let cal = aster_cal();
    assert_eq!(
        cal.date_to_instant(&date(&[100, 12, 56])),
        Err(CalendarError::InvalidValue {
            unit: s("sol"),
            value: 56,
            first: 1,
            last: 55,
            within: Some(s("Twelfth")),
            era_number: era("orbit", 100),
        })
    );
    let start = cal.date_to_instant(&date(&[500, 12, 56])).unwrap();
    assert_eq!(
        cal.instant_to_date(start.instant).unwrap().date,
        date(&[500, 12, 56, 0, 0])
    );
}

/// Scenario 8 (FR-010).
#[test]
fn convert_era_instances() {
    let cal = aster_cal();
    let mut adjusted: Vec<i64> = SPANS.iter().map(|&(_, count)| count).collect();
    let common = adjusted.clone();
    adjusted[11] = 56;
    assert_eq!(
        cal.era_instance(5),
        Ok(EraInstance {
            members: adjusted,
            length: 669,
            holding: vec![1],
        })
    );
    assert_eq!(
        cal.era_instance(1),
        Ok(EraInstance {
            members: common,
            length: 668,
            holding: vec![],
        })
    );
    assert_eq!(cal.era_instance(0).map(|y| y.length), Ok(669));
    assert_eq!(cal.era_instance(-5).map(|y| y.holding), Ok(vec![1]));
    let mut def = aster(*EPOCH);
    def.era.unit = s("sol");
    assert_eq!(
        Calendar::new(def).unwrap().era_instance(1),
        Err(CalendarError::NotAGrouping { unit: s("sol") })
    );
}

/// Units above the orbit are numbered by their count of orbits.
#[test]
fn convert_cohorts_and_ages_count_orbits() {
    let cal = aster_cal();
    let orbit_11 = cal.date_to_instant(&date(&[11])).unwrap().instant;
    assert_eq!(cal.instant_to_date(orbit_11).unwrap().above, [1, 2]);
    let last_of_10 = orbit_11.checked_add(ns(-1)).unwrap();
    let got = cal.instant_to_date(last_of_10).unwrap();
    assert_eq!(got.above, [1, 1]);
    assert_eq!(got.date.values[0], 10);
    let orbit_101 = cal.date_to_instant(&date(&[101])).unwrap().instant;
    assert_eq!(cal.instant_to_date(orbit_101).unwrap().above, [2, 11]);
}

/// Tessel with its rule: phase 3 has 13 turns in cycles divisible by 3, else 14. Cycles are 39
/// or 40 turns: cycle 1 starts at 0, 2 at 40, 3 at 80, 4 at 119; going back, cycle 0 (39) at
/// −39, −1 at −79, −2 at −119 and −3 (39) at −158.
#[test]
fn convert_tessel_with_its_rule() {
    let cal = Calendar::new(tessel_with_rule(*EPOCH)).unwrap();
    for cycle in [0, 3, -3] {
        assert_eq!(cal.era_instance(cycle).unwrap().members, [13, 13, 13]);
    }
    assert_eq!(cal.era_instance(1).unwrap().members, [13, 13, 14]);
    assert_eq!(
        cal.date_to_instant(&date(&[3, 3, 13])),
        Err(CalendarError::InvalidValue {
            unit: s("turn"),
            value: 13,
            first: 0,
            last: 12,
            within: Some(s("3")),
            era_number: era("cycle", 3),
        })
    );
    assert!(cal.date_to_instant(&date(&[1, 3, 13])).is_ok());
    for (cycle, start) in [
        (-3, -158),
        (-2, -119),
        (-1, -79),
        (0, -39),
        (1, 0),
        (2, 40),
        (3, 80),
        (4, 119),
    ] {
        assert_eq!(fixed(&cal, &[cycle]), start, "cycle {cycle}");
        let back = cal.instant_to_date(cal.start_of_fixed(start).unwrap().instant);
        assert_eq!(back.unwrap().date, date(&[cycle, 1, 0, 0]), "cycle {cycle}");
    }
    // The last turn of cycle 3 is turn 12 of phase 3, just before cycle 4.
    let last = cal.instant_to_date(cal.start_of_fixed(118).unwrap().instant);
    assert_eq!(last.unwrap().date, date(&[3, 3, 12, 0]));
}

/// Story 1 scenario 8 and FR-004: two rules that both hold add up.
#[test]
fn convert_two_rules_that_both_hold() {
    let mut def = aster_with_rule(*EPOCH);
    def.groupings[2]
        .rules
        .push(rule("orbit % 25 == 0", "Twelfth", 2));
    let cal = Calendar::new(def).unwrap();
    let orbit_25 = cal.era_instance(25).unwrap();
    assert_eq!(orbit_25.members[11], 58);
    assert_eq!(orbit_25.length, 671);
    assert_eq!(orbit_25.holding, [1, 2]);
    let start = cal.date_to_instant(&date(&[25, 12, 58])).unwrap();
    assert_eq!(
        cal.instant_to_date(start.instant).unwrap().date,
        date(&[25, 12, 58, 0, 0])
    );
    assert!(cal.date_to_instant(&date(&[25, 12, 59])).is_err());
}

/// FR-012: an orbit whose start doesn't fit is out of range, never wrapped.
#[test]
fn convert_orbits_beyond_the_range() {
    let cal = aster_cal();
    for orbit in [i64::MAX / 2, i64::MIN / 2, i64::MAX, i64::MIN] {
        assert_eq!(
            cal.date_to_instant(&date(&[orbit])),
            Err(CalendarError::OutOfRange),
            "orbit {orbit}"
        );
    }
}
