# Contract: `uni-calendar` public API

The Rust API that later features build on. `R<T>` means `Result<T, CalendarError>`. Names are fixed by this contract; doc comments, private helpers and the internal layout of `Calendar` are up to the build. Field rules are in [data-model.md](../data-model.md).

## Definition

```rust
pub enum Clock { Gct }                                   // more clocks in Phase 4 (FR-007)

pub struct CalendarDef {
    pub name: String,
    pub clock: Clock,
    pub groupings: Vec<GroupingDef>,                     // top to bottom
    pub fundamental: FundamentalDef,
    pub subdivisions: Vec<SubdivisionDef>,               // top to bottom
    pub era: EraDef,
}
pub struct FundamentalDef { pub name: String, pub length: Ratio, pub first_number: Option<i64> }
pub struct GroupingDef {
    pub name: String,
    pub member_unit: Option<MemberUnitDef>,
    pub members: Vec<MemberDef>,
    pub first_number: Option<i64>,
}
pub struct MemberUnitDef { pub name: String, pub first_number: Option<i64> }
pub struct MemberDef { pub name: Option<String>, pub count: i64 }
pub struct SubdivisionDef { pub name: String, pub parts: i64, pub first_number: Option<i64> }
pub struct EraDef { pub name: String, pub unit: String, pub epoch: Instant }
// all derive: Clone, Debug, PartialEq, Eq
```

## Calendar

```rust
impl Calendar {
    pub fn new(def: CalendarDef) -> Result<Calendar, DefinitionError>;  // checks everything (FR-006)
    pub fn name(&self) -> &str;
    pub fn clock(&self) -> Clock;
    pub fn era_name(&self) -> &str;
    pub fn epoch(&self) -> Instant;
    pub fn units(&self) -> Vec<&str>;                    // the chain, top to bottom, member units included
    pub fn era_unit(&self) -> &str;
    pub fn units_per(&self, outer: &str, inner: &str) -> R<i64>;  // year in sols = 668; sol in minutes = 2000;
                                                                   // NotFixed when it varies (month in sols)

    pub fn fixed_from_instant(&self, instant: Instant) -> R<i64>;           // FR-012
    pub fn start_of_fixed(&self, fixed: i64) -> R<DateStart>;               // FR-012
    pub fn instant_to_date(&self, instant: Instant) -> R<InstantDate>;      // FR-010
    pub fn date_to_instant(&self, date: &Date) -> R<DateStart>;             // FR-011, FR-013, FR-014
}
// Calendar derives: Clone, Debug, PartialEq, Eq
```

## Dates

```rust
pub struct Date { pub values: Vec<i64> }                 // from the era's unit downwards
pub struct InstantDate { pub above: Vec<i64>, pub date: Date, pub remainder: Ratio }
pub struct DateStart { pub instant: Instant, pub rounded_by: Option<Ratio> }
// all derive: Clone, Debug, PartialEq, Eq, Hash
```

## Errors

```rust
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
pub enum Limit { LengthTooFine, ShorterThanNanosecond, TooManyTicks, GroupingTooLong }

pub enum CalendarError {
    InvalidValue { unit: String, value: i64, first: i64, last: i64, within: Option<String> },
    OutOfRange,
    EmptyDate,
    TooManyValues { given: usize, max: usize },
    UnknownUnit { name: String },
    NotFixed { unit: String, of: String },
}
// both: Clone, Debug, PartialEq, Eq; Display (messages in data-model.md) + std::error::Error
```
