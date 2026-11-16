# Contract: `uni-calendar` public API, changes for leap rules

Changes to spec 004's [contract](../../004-calendar-engine-basics/contracts/uni-calendar-api.md); everything not listed stays as it is. `R<T>` means `Result<T, CalendarError>`. Field rules and messages are in [data-model.md](../data-model.md).

## Dependency

`uni-calendar` depends on `uni-expr` (allowed by `/architecture/crate-layout.md`). Its errors carry `uni_expr::CheckError`, `PeriodError` and `EvalError` as they are.

## Definition

```rust
pub struct GroupingDef {
    pub name: String,
    pub member_unit: Option<MemberUnitDef>,
    pub members: Vec<MemberDef>,
    pub first_number: Option<i64>,
    pub rules: Vec<LeapRuleDef>,                         // new; only on the era's unit
}
pub struct LeapRuleDef {                                 // new
    pub condition: String,                               // e.g. "year % 4 == 0"
    pub member: String,                                  // name, or number from 1
    pub change: i64,                                     // ≠ 0; + lengthens, − shortens
}
// all derive: Clone, Debug, PartialEq, Eq
```

## Calendar

```rust
pub const RULE_PERIOD_LIMIT: i64 = 100_000;              // research R3

impl Calendar {
    /// The number of instances of the era's unit after which its rules repeat (1 without
    /// rules). Aster: 500.
    pub fn rule_period(&self) -> i64;
    /// One instance of the era's unit after its rules (FR-010).
    pub fn era_instance(&self, number: i64) -> R<EraInstance>;
    // unchanged signatures; behaviour follows the rules (FR-009 to FR-012):
    // units_per, fixed_from_instant, start_of_fixed, instant_to_date, date_to_instant
}

pub struct EraInstance {                                 // new
    pub members: Vec<i64>,                               // counts after the rules
    pub length: i64,                                     // in fundamental units
    pub holding: Vec<usize>,                             // rule numbers from 1
}
// derives: Clone, Debug, PartialEq, Eq, Hash
```

## Errors

```rust
pub enum DefinitionError {
    // … spec 004's variants, plus:
    RulesNotOnEraUnit { unit: String },
    RuleCondition { unit: String, rule: usize, error: uni_expr::CheckError },
    RuleUnknownMember { unit: String, rule: usize, member: String },
    RuleZeroChange { unit: String, rule: usize },
    RulePeriod { unit: String, rule: usize, error: uni_expr::PeriodError },
    RulesPeriodOverLimit { unit: String, limit: i64 },
    RuleFails { unit: String, rule: usize, year: i64, error: uni_expr::EvalError },
    MemberTooShort { unit: String, member: String, year: i64, length: i64 },
}
pub enum Limit { LengthTooFine, ShorterThanNanosecond, TooManyTicks, GroupingTooLong, CycleTooLong }

pub enum CalendarError {
    InvalidValue { unit: String, value: i64, first: i64, last: i64, within: Option<String>, year: Option<i64> },
    // … OutOfRange, EmptyDate, TooManyValues, UnknownUnit, NotFixed as before, plus:
    NotAGrouping { unit: String },
}
```

## Example

```rust
// Aster (spec 004) with its leap rule on the year.
let mut def = aster();
def.groupings[2].rules = vec![LeapRuleDef {
    condition: "year % 5 == 0 && (year % 100 != 0 || year % 500 == 0)".into(),
    member: "Twelfth".into(),
    change: 1,
}];
let cal = Calendar::new(def)?;
assert_eq!(cal.rule_period(), 500);
assert_eq!(cal.era_instance(5)?.length, 669);
assert_eq!(cal.units_per("year", "month"), Ok(12));
assert!(matches!(cal.units_per("year", "sol"), Err(CalendarError::NotFixed { .. })));
let start = cal.date_to_instant(&Date { values: vec![6] })?;    // 3,341 sols after the epoch
assert_eq!(cal.fixed_from_instant(start.instant)?, 3_341);
```
