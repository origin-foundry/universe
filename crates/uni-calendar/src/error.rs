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
            } => {
                write!(f, "{unit} {value} is outside {first} to {last}")?;
                match within {
                    Some(member) => write!(f, " in {member}"),
                    None => Ok(()),
                }
            }
            E::OutOfRange => f.write_str("date is out of the supported range"),
            E::EmptyDate => f.write_str("a date needs at least the era's unit"),
            E::TooManyValues { max, .. } => write!(f, "a date has at most {max} values here"),
            E::UnknownUnit { name } => write!(f, "no unit named {name}"),
            E::NotFixed { unit, of } => write!(f, "the number of {of}s in a {unit} varies"),
        }
    }
}

impl std::error::Error for CalendarError {}

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
            (E::NoMembers { unit: s("year") }, "year has no members"),
            (
                E::NonPositiveCount {
                    unit: s("year"),
                    member: s("Third"),
                },
                "member Third of year must have a positive count",
            ),
            (
                E::MembersNeedUnit { unit: s("year") },
                "year has more than one member, so it needs a member unit",
            ),
            (
                E::TooFewParts { unit: s("hour") },
                "hour must have at least one part",
            ),
            (
                E::DuplicateUnit { name: s("sol") },
                "two units are named sol",
            ),
            (
                E::DuplicateMember {
                    unit: s("year"),
                    name: s("Prime"),
                },
                "year has two members named Prime",
            ),
            (
                E::EraUnitUnknown { unit: s("week") },
                "the era numbers week, which isn't a unit of this calendar",
            ),
            (
                E::EraUnitBelowFundamental { unit: s("hour") },
                "the era can't number hour, a subdivision of the fundamental unit",
            ),
            (
                E::FirstNumberNotAllowed { unit: s("decade") },
                "decade can't have a first number: the era numbers it",
            ),
            (
                E::BeyondLimits {
                    unit: s("minute"),
                    limit: Limit::ShorterThanNanosecond,
                },
                "minute is beyond the engine's limits: it is shorter than a nanosecond",
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
                },
                "sol 56 is outside 1 to 55 in Third",
            ),
            (
                E::InvalidValue {
                    unit: s("hour"),
                    value: 20,
                    first: 0,
                    last: 19,
                    within: None,
                },
                "hour 20 is outside 0 to 19",
            ),
            (E::OutOfRange, "date is out of the supported range"),
            (E::EmptyDate, "a date needs at least the era's unit"),
            (
                E::TooManyValues { given: 6, max: 5 },
                "a date has at most 5 values here",
            ),
            (E::UnknownUnit { name: s("week") }, "no unit named week"),
            (
                E::NotFixed {
                    unit: s("month"),
                    of: s("sol"),
                },
                "the number of sols in a month varies",
            ),
        ];
        for (error, message) in cases {
            assert_eq!(error.to_string(), message);
        }
    }

    #[test]
    fn errors_are_std_errors() {
        fn takes(_: &dyn std::error::Error) {}
        takes(&CalendarError::OutOfRange);
        takes(&DefinitionError::NoMembers { unit: s("year") });
    }
}
