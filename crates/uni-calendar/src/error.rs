use std::fmt;

/// Why a calendar definition was refused (FR-006). Each variant names the unit, and the member
/// where there is one, at fault.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefinitionError {
    /// The calendar, the era, a unit or a member has an empty name; `what` says which.
    EmptyName { what: &'static str },
    /// The fundamental unit's length is zero or negative (FR-002).
    NonPositiveLength { unit: String },
    /// A grouping lists no members (FR-003).
    NoMembers { unit: String },
    /// A member's count is zero or negative; `member` is its name, or its number from 1 when
    /// it has none (FR-003).
    NonPositiveCount { unit: String, member: String },
    /// A grouping has more than one member but no member unit (research R1).
    MembersNeedUnit { unit: String },
    /// A subdivision has fewer than one part (FR-004).
    TooFewParts { unit: String },
    /// Two units share a name (FR-005).
    DuplicateUnit { name: String },
    /// Two members of one grouping share a name (FR-005).
    DuplicateMember { unit: String, name: String },
    /// The era names a unit that isn't in the calendar (FR-006).
    EraUnitUnknown { unit: String },
    /// The era names a subdivision (FR-006).
    EraUnitBelowFundamental { unit: String },
    /// A first number is given for the era's unit or a unit above it, which the era numbers
    /// (research R5).
    FirstNumberNotAllowed { unit: String },
    /// One of the engine's limits (research R3) is exceeded.
    BeyondLimits { unit: String, limit: Limit },
    /// A grouping other than the era's unit has adjustment rules, or the era's unit has them but
    /// isn't a grouping (spec 006, research R5).
    RulesNotOnEraUnit { unit: String },
    /// Rule `rule` (from 1) of `unit` has a condition the expression language refuses,
    /// including an era unit name that can't be a variable name.
    RuleCondition {
        unit: String,
        rule: usize,
        error: uni_expr::CheckError,
    },
    /// Rule `rule` changes a member `unit` doesn't have.
    RuleUnknownMember {
        unit: String,
        rule: usize,
        member: String,
    },
    /// Rule `rule` changes its member by 0.
    RuleZeroChange { unit: String, rule: usize },
    /// Rule `rule`'s condition doesn't repeat, or repeats only beyond the limit.
    RulePeriod {
        unit: String,
        rule: usize,
        error: uni_expr::PeriodError,
    },
    /// The rules together repeat only after more than `limit` instances.
    RulesPeriodOverLimit { unit: String, limit: i64 },
    /// Rule `rule`'s condition fails to evaluate for instance `era_number` of `unit`.
    RuleFails {
        unit: String,
        rule: usize,
        era_number: i64,
        error: uni_expr::EvalError,
    },
    /// `member` of `unit` would be `length` long (below 1) in instance `era_number`.
    MemberTooShort {
        unit: String,
        member: String,
        era_number: i64,
        length: i64,
    },
}

/// The engine's limits (research R3), so a valid calendar never fails part-way through the
/// range it promises.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    /// The fundamental unit's length in nanoseconds has a denominator over 10⁹.
    LengthTooFine,
    /// The smallest unit is shorter than one nanosecond.
    ShorterThanNanosecond,
    /// The fundamental unit is too long for how finely it's subdivided: 2¹²⁰ ticks or more.
    TooManyTicks,
    /// A grouping's length in fundamental units doesn't fit in 64 bits.
    GroupingTooLong,
    /// One repetition of the era's unit's rules doesn't fit in 64 bits of fundamental units.
    CycleTooLong,
}

/// An instance of the era's unit, named so a message can say which one: "orbit 1",
/// "cycle 3".
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EraNumber {
    /// The era's unit's name.
    pub unit: String,
    /// The instance's astronomical number, as the date shows it.
    pub number: i64,
}

/// Why a conversion or query was refused (FR-013, FR-015, FR-019).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CalendarError {
    /// A date value is outside its unit's range, `first` to `last`; `within` names the member
    /// (by name, or number from 1) when the range depends on it.
    InvalidValue {
        unit: String,
        value: i64,
        first: i64,
        last: i64,
        within: Option<String>,
        /// The instance of the era's unit the value was checked against: set whenever the
        /// calendar has adjustment rules and the unit is below the era's unit, `None` otherwise.
        era_number: Option<EraNumber>,
    },
    /// The result needs a number outside 64 bits, or an instant outside the `Instant` range.
    OutOfRange,
    /// A date with no values.
    EmptyDate,
    /// More values than there are units from the era's unit down.
    TooManyValues { given: usize, max: usize },
    /// A query names a unit that isn't in the calendar.
    UnknownUnit { name: String },
    /// `units_per` asked for a count that isn't one fixed whole number: how many `of` are in
    /// a `unit`.
    NotFixed { unit: String, of: String },
    /// `era_instance` on a calendar whose era's unit isn't a grouping.
    NotAGrouping { unit: String },
}

impl fmt::Display for DefinitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DefinitionError as E;
        match self {
            E::EmptyName { what } => write!(f, "the {what} needs a name"),
            E::NonPositiveLength { unit } => write!(f, "{unit} must have a positive length"),
            E::NoMembers { unit } => write!(f, "{unit} has no members"),
            E::NonPositiveCount { unit, member } => {
                write!(f, "member {member} of {unit} must have a positive count")
            }
            E::MembersNeedUnit { unit } => {
                write!(
                    f,
                    "{unit} has more than one member, so it needs a member unit"
                )
            }
            E::TooFewParts { unit } => write!(f, "{unit} must have at least one part"),
            E::DuplicateUnit { name } => write!(f, "two units are named {name}"),
            E::DuplicateMember { unit, name } => write!(f, "{unit} has two members named {name}"),
            E::EraUnitUnknown { unit } => {
                write!(
                    f,
                    "the era numbers {unit}, which isn't a unit of this calendar"
                )
            }
            E::EraUnitBelowFundamental { unit } => {
                write!(
                    f,
                    "the era can't number {unit}, a subdivision of the fundamental unit"
                )
            }
            E::FirstNumberNotAllowed { unit } => {
                write!(f, "{unit} can't have a first number: the era numbers it")
            }
            E::BeyondLimits { unit, limit } => {
                write!(f, "{unit} is beyond the engine's limits: {limit}")
            }
            E::RulesNotOnEraUnit { unit } => write!(
                f,
                "{unit} can't have adjustment rules: rules go on the era's unit for now"
            ),
            E::RuleCondition { unit, rule, error } => write!(f, "rule {rule} of {unit}: {error}"),
            E::RuleUnknownMember { unit, rule, member } => write!(
                f,
                "rule {rule} of {unit} changes {member}, which {unit} doesn't have"
            ),
            E::RuleZeroChange { unit, rule } => {
                write!(f, "rule {rule} of {unit} changes its member by 0")
            }
            E::RulePeriod { unit, rule, error } => write!(f, "rule {rule} of {unit}: {error}"),
            E::RulesPeriodOverLimit { unit, limit } => write!(
                f,
                "the rules of {unit} repeat only after more than {} {unit}s",
                grouped(*limit)
            ),
            E::RuleFails {
                unit,
                rule,
                era_number,
                error,
            } => write!(
                f,
                "rule {rule} of {unit} fails in {unit} {era_number}: {error}"
            ),
            E::MemberTooShort {
                unit,
                member,
                era_number,
                length,
            } => write!(
                f,
                "{member} of {unit} would be {length} long in {unit} {era_number}"
            ),
        }
    }
}

impl std::error::Error for DefinitionError {}

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Limit::LengthTooFine => "its length in nanoseconds has a denominator over 10⁹",
            Limit::ShorterThanNanosecond => "it is shorter than a nanosecond",
            Limit::TooManyTicks => "it is too long for how finely it is subdivided",
            Limit::GroupingTooLong => "it is too long to count in fundamental units",
            Limit::CycleTooLong => {
                "one repetition of its rules is too long to count in fundamental units"
            }
        })
    }
}

impl fmt::Display for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CalendarError as E;
        match self {
            E::InvalidValue {
                unit,
                value,
                first,
                last,
                within,
                era_number,
            } => {
                write!(f, "{unit} {value} is outside {first} to {last}")?;
                if let Some(member) = within {
                    write!(f, " in {member}")?;
                }
                match era_number {
                    Some(EraNumber { unit, number }) => write!(f, " in {unit} {number}"),
                    None => Ok(()),
                }
            }
            E::OutOfRange => f.write_str("date is out of the supported range"),
            E::EmptyDate => f.write_str("a date needs at least the era's unit"),
            E::TooManyValues { max, .. } => write!(f, "a date has at most {max} values here"),
            E::UnknownUnit { name } => write!(f, "no unit named {name}"),
            E::NotFixed { unit, of } => write!(f, "the number of {of}s in a {unit} varies"),
            E::NotAGrouping { unit } => {
                write!(f, "{unit} isn't a grouping, so it has no members")
            }
        }
    }
}

impl std::error::Error for CalendarError {}

/// A whole number with commas between groups of three digits.
fn grouped(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    if n < 0 {
        out.push('-');
    }
    for (i, d) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(d);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(text: &str) -> String {
        text.to_string()
    }

    #[test]
    fn definition_messages_name_the_fault() {
        use DefinitionError as E;
        let cases = [
            (E::EmptyName { what: "era" }, "the era needs a name"),
            (
                E::NonPositiveLength { unit: s("sol") },
                "sol must have a positive length",
            ),
            (E::NoMembers { unit: s("orbit") }, "orbit has no members"),
            (
                E::NonPositiveCount {
                    unit: s("orbit"),
                    member: s("Third"),
                },
                "member Third of orbit must have a positive count",
            ),
            (
                E::MembersNeedUnit { unit: s("orbit") },
                "orbit has more than one member, so it needs a member unit",
            ),
            (
                E::TooFewParts { unit: s("bell") },
                "bell must have at least one part",
            ),
            (
                E::DuplicateUnit { name: s("sol") },
                "two units are named sol",
            ),
            (
                E::DuplicateMember {
                    unit: s("orbit"),
                    name: s("Prime"),
                },
                "orbit has two members named Prime",
            ),
            (
                E::EraUnitUnknown { unit: s("rota") },
                "the era numbers rota, which isn't a unit of this calendar",
            ),
            (
                E::EraUnitBelowFundamental { unit: s("bell") },
                "the era can't number bell, a subdivision of the fundamental unit",
            ),
            (
                E::FirstNumberNotAllowed { unit: s("cohort") },
                "cohort can't have a first number: the era numbers it",
            ),
            (
                E::BeyondLimits {
                    unit: s("pulse"),
                    limit: Limit::ShorterThanNanosecond,
                },
                "pulse is beyond the engine's limits: it is shorter than a nanosecond",
            ),
        ];
        for (error, message) in cases {
            assert_eq!(error.to_string(), message);
        }
    }

    #[test]
    fn rule_messages_name_the_rule_and_orbit() {
        use DefinitionError as E;
        use uni_expr::{CheckError, EvalError, PeriodError, Pos};
        let at = |column| Pos { line: 1, column };
        let cases = [
            (
                E::RulesNotOnEraUnit { unit: s("cohort") },
                "cohort can't have adjustment rules: rules go on the era's unit for now",
            ),
            (
                E::RuleCondition {
                    unit: s("orbit"),
                    rule: 1,
                    error: CheckError::UnknownVariable {
                        name: s("span"),
                        at: at(1),
                    },
                },
                "rule 1 of orbit: unknown variable `span` at 1:1",
            ),
            (
                E::RuleUnknownMember {
                    unit: s("orbit"),
                    rule: 1,
                    member: s("Thirteenth"),
                },
                "rule 1 of orbit changes Thirteenth, which orbit doesn't have",
            ),
            (
                E::RuleZeroChange {
                    unit: s("orbit"),
                    rule: 2,
                },
                "rule 2 of orbit changes its member by 0",
            ),
            (
                E::RulePeriod {
                    unit: s("orbit"),
                    rule: 1,
                    error: PeriodError::NotRepeating { at: at(6) },
                },
                "rule 1 of orbit: the part at 1:6 stops the expression repeating as the variable \
                 counts up",
            ),
            (
                E::RulesPeriodOverLimit {
                    unit: s("orbit"),
                    limit: 100_000,
                },
                "the rules of orbit repeat only after more than 100,000 orbits",
            ),
            (
                E::RuleFails {
                    unit: s("orbit"),
                    rule: 1,
                    era_number: 7,
                    error: EvalError::DivisionByZero { at: at(5) },
                },
                "rule 1 of orbit fails in orbit 7: division by zero at 1:5",
            ),
            (
                E::MemberTooShort {
                    unit: s("orbit"),
                    member: s("Third"),
                    era_number: 2,
                    length: 0,
                },
                "Third of orbit would be 0 long in orbit 2",
            ),
            (
                E::BeyondLimits {
                    unit: s("orbit"),
                    limit: Limit::CycleTooLong,
                },
                "orbit is beyond the engine's limits: one repetition of its rules is too long to \
                 count in fundamental units",
            ),
        ];
        for (error, message) in cases {
            assert_eq!(error.to_string(), message);
        }
    }

    #[test]
    fn limit_messages() {
        assert_eq!(
            Limit::LengthTooFine.to_string(),
            "its length in nanoseconds has a denominator over 10⁹"
        );
        assert_eq!(
            Limit::TooManyTicks.to_string(),
            "it is too long for how finely it is subdivided"
        );
        assert_eq!(
            Limit::GroupingTooLong.to_string(),
            "it is too long to count in fundamental units"
        );
    }

    #[test]
    fn calendar_messages_name_the_unit_and_range() {
        use CalendarError as E;
        let cases = [
            (
                E::InvalidValue {
                    unit: s("sol"),
                    value: 56,
                    first: 1,
                    last: 55,
                    within: Some(s("Third")),
                    era_number: None,
                },
                "sol 56 is outside 1 to 55 in Third",
            ),
            (
                E::InvalidValue {
                    unit: s("bell"),
                    value: 20,
                    first: 0,
                    last: 19,
                    within: None,
                    era_number: None,
                },
                "bell 20 is outside 0 to 19",
            ),
            (
                E::InvalidValue {
                    unit: s("sol"),
                    value: 56,
                    first: 1,
                    last: 55,
                    within: Some(s("Twelfth")),
                    era_number: Some(EraNumber {
                        unit: s("orbit"),
                        number: 1,
                    }),
                },
                "sol 56 is outside 1 to 55 in Twelfth in orbit 1",
            ),
            (
                E::InvalidValue {
                    unit: s("turn"),
                    value: 13,
                    first: 0,
                    last: 12,
                    within: Some(s("3")),
                    era_number: Some(EraNumber {
                        unit: s("cycle"),
                        number: 3,
                    }),
                },
                "turn 13 is outside 0 to 12 in 3 in cycle 3",
            ),
            (
                E::NotAGrouping { unit: s("sol") },
                "sol isn't a grouping, so it has no members",
            ),
            (E::OutOfRange, "date is out of the supported range"),
            (E::EmptyDate, "a date needs at least the era's unit"),
            (
                E::TooManyValues { given: 6, max: 5 },
                "a date has at most 5 values here",
            ),
            (E::UnknownUnit { name: s("rota") }, "no unit named rota"),
            (
                E::NotFixed {
                    unit: s("span"),
                    of: s("sol"),
                },
                "the number of sols in a span varies",
            ),
        ];
        for (error, message) in cases {
            assert_eq!(error.to_string(), message);
        }
    }

    #[test]
    fn grouping_digits() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(100_000), "100,000");
        assert_eq!(grouped(-1_234_567), "-1,234,567");
    }

    #[test]
    fn errors_are_std_errors() {
        fn takes(_: &dyn std::error::Error) {}
        takes(&CalendarError::OutOfRange);
        takes(&DefinitionError::NoMembers { unit: s("orbit") });
    }
}
