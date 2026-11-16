//! Hand-worked tests for leap rules (spec 006): defining calendars with rules (`define_`,
//! Story 1) and converting dates in them (`convert_`, Story 2), on Aster and Tessel.

mod common;

use common::*;
use uni_calendar::{Calendar, CalendarDef, CalendarError, DefinitionError, LeapRuleDef};
use uni_expr::{CheckError, EvalError, PeriodError, Pos};

fn s(text: &str) -> String {
    text.to_string()
}

fn at(column: u32) -> Pos {
    Pos { line: 1, column }
}

/// Aster with `rules` on its year.
fn aster_rules(rules: Vec<LeapRuleDef>) -> CalendarDef {
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
// Story 1: define a calendar with leap rules

/// Scenario 1.
#[test]
fn define_aster_with_its_rule_is_accepted() {
    let cal = Calendar::new(aster_with_rule(*EPOCH)).unwrap();
    assert_eq!(cal.rule_period(), 500);
    assert_eq!(cal.units_per("year", "sol"), not_fixed("year", "sol"));
    assert_eq!(cal.units_per("decade", "sol"), not_fixed("decade", "sol"));
    assert_eq!(cal.units_per("year", "month"), Ok(12));
    assert_eq!(cal.units_per("decade", "year"), Ok(10));
    assert_eq!(cal.units_per("age", "month"), Ok(1_200));
    // Inside a sol nothing changes.
    assert_eq!(cal.units_per("sol", "minute"), Ok(2_000));

    let plain = Calendar::new(aster(*EPOCH)).unwrap();
    assert_eq!(plain.rule_period(), 1);
    assert_eq!(plain.units_per("year", "sol"), Ok(668));
}

/// Scenario 2.
#[test]
fn define_conditions_the_expression_language_refuses() {
    match refused(aster_rules(vec![rule("year % 4", "Twelfth", 1)])) {
        DefinitionError::RuleCondition {
            unit,
            rule: 1,
            error: CheckError::TypeMismatch {
                context: "result", ..
            },
        } => assert_eq!(unit, "year"),
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(
        refused(aster_rules(vec![rule("month == 2", "Twelfth", 1)])),
        DefinitionError::RuleCondition {
            unit: s("year"),
            rule: 1,
            error: CheckError::UnknownVariable {
                name: s("month"),
                at: at(1)
            },
        }
    );
}

/// Scenario 3.
#[test]
fn define_unknown_members_and_zero_changes() {
    assert_eq!(
        refused(aster_rules(vec![rule("year % 4 == 0", "Thirteenth", 1)])),
        DefinitionError::RuleUnknownMember {
            unit: s("year"),
            rule: 1,
            member: s("Thirteenth"),
        }
    );
    assert_eq!(
        refused(aster_rules(vec![rule("year % 4 == 0", "Twelfth", 0)])),
        DefinitionError::RuleZeroChange {
            unit: s("year"),
            rule: 1
        }
    );
    // A second rule's fault names rule 2.
    assert_eq!(
        refused(aster_rules(vec![
            rule("year % 4 == 0", "Twelfth", 1),
            rule("year % 7 == 0", "Third", 0),
        ])),
        DefinitionError::RuleZeroChange {
            unit: s("year"),
            rule: 2
        }
    );
}

/// Scenario 4.
#[test]
fn define_members_shortened_to_nothing() {
    assert_eq!(
        refused(aster_rules(vec![rule("year % 2 == 0", "Third", -55)])),
        DefinitionError::MemberTooShort {
            unit: s("year"),
            member: s("Third"),
            year: 2,
            length: 0,
        }
    );
    // Changes add up: 55 − 30 − 30.
    assert_eq!(
        refused(aster_rules(vec![
            rule("year % 2 == 0", "Third", -30),
            rule("year % 2 == 0", "Third", -30),
        ])),
        DefinitionError::MemberTooShort {
            unit: s("year"),
            member: s("Third"),
            year: 2,
            length: -5,
        }
    );
}

/// Scenario 5.
#[test]
fn define_conditions_that_fail_for_some_year() {
    // Year 7 is the first where `year % 7` is 0; `/` is at column 5.
    assert_eq!(
        refused(aster_rules(vec![rule(
            "100 / (year % 7) == 3",
            "Twelfth",
            1
        )])),
        DefinitionError::RuleFails {
            unit: s("year"),
            rule: 1,
            year: 7,
            error: EvalError::DivisionByZero { at: at(5) },
        }
    );
    // 2 × 2⁶² overflows: year 2.
    match refused(aster_rules(vec![rule(
        "year * 4611686018427387904 % 3 == 0",
        "Twelfth",
        1,
    )])) {
        DefinitionError::RuleFails {
            unit,
            rule: 1,
            year: 2,
            error: EvalError::OutOfRange { .. },
        } => assert_eq!(unit, "year"),
        other => panic!("unexpected {other:?}"),
    }
}

/// Scenario 6.
#[test]
fn define_rules_only_on_the_era_unit() {
    let mut def = aster(*EPOCH);
    def.groupings[1].rules = vec![rule("decade % 2 == 0", "1", 1)];
    assert_eq!(
        refused(def),
        DefinitionError::RulesNotOnEraUnit { unit: s("decade") }
    );
    // An era that numbers sols: the year isn't the era's unit.
    let mut def = aster_with_rule(*EPOCH);
    def.era.unit = s("sol");
    assert_eq!(
        refused(def),
        DefinitionError::RulesNotOnEraUnit { unit: s("year") }
    );
}

/// Scenario 7.
#[test]
fn define_rules_that_dont_repeat_within_the_limit() {
    assert_eq!(
        refused(aster_rules(vec![rule("year > 100", "Twelfth", 1)])),
        DefinitionError::RulePeriod {
            unit: s("year"),
            rule: 1,
            error: PeriodError::NotRepeating { at: at(6) },
        }
    );
    assert_eq!(
        refused(aster_rules(vec![rule("year % 100003 == 0", "Twelfth", 1)])),
        DefinitionError::RulePeriod {
            unit: s("year"),
            rule: 1,
            error: PeriodError::OverLimit { limit: 100_000 },
        }
    );
    // Each is fine, but together they repeat every 97 × 89 × 83 = 716,539 years.
    assert_eq!(
        refused(aster_rules(vec![
            rule("year % 97 == 0", "Twelfth", 1),
            rule("year % 89 == 0", "Twelfth", 1),
            rule("year % 83 == 0", "Twelfth", 1),
        ])),
        DefinitionError::RulesPeriodOverLimit {
            unit: s("year"),
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
