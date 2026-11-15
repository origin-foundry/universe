use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefinitionError {
    EmptyName { what: &'static str },
    NonPositiveLength { unit: String },
    NoMembers { unit: String },
    NonPositiveCount { unit: String, member: String },
    MembersNeedUnit { unit: String },
    TooFewParts { unit: String },
    DuplicateUnit { name: String },
    DuplicateMember { unit: String, name: String },
    EraUnitUnknown { unit: String },
    EraUnitBelowFundamental { unit: String },
    FirstNumberNotAllowed { unit: String },
    BeyondLimits { unit: String, limit: Limit },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    LengthTooFine,
    ShorterThanNanosecond,
    TooManyTicks,
    GroupingTooLong,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CalendarError {
    InvalidValue {
        unit: String,
        value: i64,
        first: i64,
        last: i64,
        within: Option<String>,
    },
    OutOfRange,
    EmptyDate,
    TooManyValues { given: usize, max: usize },
    UnknownUnit { name: String },
    NotFixed { unit: String, of: String },
}

impl fmt::Display for DefinitionError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for DefinitionError {}

impl fmt::Display for Limit {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl fmt::Display for CalendarError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for CalendarError {}
