# Contract: calendars in `uni-time`, `uni-store` and the app's commands

Additions only; spec 003's contract (`specs/003-project-file/contracts/store-and-commands.md`) is unchanged.

## `uni-time` (Rust)

```rust
impl Ratio {
    /// `[+-]digits`, `[+-]digits.digits` or `[+-]digits/digits`; no spaces, exponent or separators.
    /// Exact, never rounded.
    pub fn parse(text: &str) -> Result<Ratio, TimeError>;
}

pub enum TimeError { OutOfRange, DivisionByZero, Overflow, NotANumber /* new */ }
// NotANumber: "not a whole number, decimal or fraction"
```

## `uni-store` (Rust)

```rust
pub const SCHEMA_VERSION: u32 = 2;     // MIGRATIONS gains 1 → 2 (research R2)

pub struct CalendarRecord {
    pub name: String,
    pub groupings: Vec<uni_calendar::GroupingDef>,
    pub fundamental: FundamentalRecord,
    pub subdivisions: Vec<uni_calendar::SubdivisionDef>,
    pub era: EraRecord,
}
pub struct FundamentalRecord { pub name: String, pub length: String, pub first_number: Option<i64> }
pub struct EraRecord { pub name: String, pub unit: String }

impl CalendarRecord {
    /// Parses the length; the epoch is the fixed hidden point (Instant::ZERO) until spec 008.
    pub fn to_def(&self) -> Result<uni_calendar::CalendarDef, RecordError>;
}
pub enum RecordError { Length { text: String, error: uni_time::TimeError } }
// Display: "the fundamental unit's length \"<text>\" is <error>"

pub struct StoredCalendar { pub id: i64, pub record: CalendarRecord }

impl OpenUniverse {
    pub fn calendars(&self) -> Result<Vec<StoredCalendar>, StoreError>;
    pub fn calendar(&self, id: i64) -> Result<StoredCalendar, StoreError>;
    pub fn add_calendar(&mut self, record: &CalendarRecord) -> Result<i64, StoreError>;
    pub fn replace_calendar(&mut self, id: i64, record: &CalendarRecord) -> Result<(), StoreError>;
    pub fn delete_calendar(&mut self, id: i64) -> Result<(), StoreError>;
}

pub enum StoreError {
    /* spec 003's variants, plus: */
    CalendarName,                     // "A calendar needs a name of 1 to 200 characters."
    CalendarNameTaken { name: String }, // "This universe already has a calendar called <name>."
    NoSuchCalendar { id: i64 },       // "That calendar no longer exists."
}
```

`uni-store`'s `Cargo.toml` gains `uni-calendar` and `uni-time` (both allowed by `/architecture/crate-layout.md`) and `publish = false`. Dev-dependency: `proptest`.

## Tauri commands (`uni-app`)

| Command | Arguments | Returns | Errors |
|---|---|---|---|
| `list_calendars` | none | `CalendarSummary[]` | string (store error) |
| `get_calendar` | `id: number` | `CalendarForm` | string |
| `new_calendar_form` | none | `CalendarForm` (research R7) | string |
| `put_calendar` | `id: number \| null`, `form: CalendarForm` | `{ state: UniverseState, id: number }` | `FieldError` (research R8), or string for a store failure |
| `delete_calendar` | `id: number` | `UniverseState` | string |

Every command except `new_calendar_form` needs an open universe; with none, it returns the string "No universe is open." Shapes are in [data-model.md](../data-model.md).

```ts
type Location =
  | { kind: "calendarName" }
  | { kind: "era"; field: "name" | "unit" }
  | { kind: "grouping"; index: number; field?: "name" | "firstNumber" | "memberUnit" }
  | { kind: "member"; grouping: number; index: number; field?: "name" | "count" }
  | { kind: "rule"; grouping: number; index: number; field?: "condition" | "member" | "change" }
  | { kind: "fundamental"; field?: "name" | "length" | "firstNumber" }
  | { kind: "subdivision"; index: number; field?: "name" | "parts" | "firstNumber" };
interface FieldError { message: string; at: Location }
interface CalendarSummary { id: number; name: string; problem: string | null }
```

The UI tells a `FieldError` from a plain store error by its shape (an object with `at`).

## Where each `DefinitionError` is placed

| Engine error | Location |
|---|---|
| `EmptyName { what }` | the named thing's `name` field (calendar, era, unit or member, by `what` and the first empty one in form order) |
| `NonPositiveLength` | `fundamental.length` |
| `NoMembers { unit }` | that `grouping` |
| `NonPositiveCount { unit, member }` | that `member.count` |
| `MembersNeedUnit { unit }` | that `grouping.memberUnit` |
| `TooFewParts { unit }` | that `subdivision.parts` |
| `DuplicateUnit { name }` | the second unit (or member unit) with that name, its `name` |
| `DuplicateMember { unit, name }` | the second such member's `name` |
| `EraUnitUnknown`, `EraUnitBelowFundamental` | `era.unit` |
| `FirstNumberNotAllowed { unit }` | that unit's `firstNumber` (a member unit's is on its grouping, `memberUnit`) |
| `BeyondLimits { unit, .. }` | that unit; `fundamental.length` when the unit is the fundamental unit |
| `RulesNotOnEraUnit { unit }` | that `grouping` |
| `RuleCondition`, `RulePeriod`, `RuleFails` | that `rule.condition` |
| `RuleUnknownMember` | that `rule.member` |
| `RuleZeroChange` | that `rule.change` |
| `RulesPeriodOverLimit { unit }` | that `grouping` |
| `MemberTooShort { unit, member }` | that `member.count` |

A unit name that matches no unit in the form (which can't happen for a form the engine just read) falls back to `{ kind: "calendarName" }`. That arm is covered by a test and never panics.
