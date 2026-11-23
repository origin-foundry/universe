# Data Model: Date Formats

Types are named as they will be in code; Rust in `crates/`, TypeScript in `ui/src`. Decisions are in [research.md](research.md).

## `uni-format`

```rust
/// A pattern compiled against one calendar (R1, R4).
pub struct Format { parts: Vec<Part>, readable: Result<Run, WriteOnly> }

enum Part {
    Literal(String),               // braces already undoubled
    Space,                         // a run of whitespace in the pattern
    Number { unit: usize, width: Option<u8> },   // index in Calendar::units()
    Name { unit: usize },          // a member unit's member name
    Era,
}

/// Why a format can't read text (R5).
pub enum WriteOnly {
    Missing { unit: String },               // the era's unit isn't shown
    Gap { shown: String, missing: String }, // a unit between two shown ones isn't
}

/// Why a pattern is refused (R4). `at` is the character position from 0.
pub struct PatternError { pub at: usize, pub kind: PatternErrorKind }
pub enum PatternErrorKind {
    Unclosed, StrayBrace, EmptyField,
    UnknownUnit { name: String },
    UnknownModifier { modifier: String },
    BadWidth { width: String },
    NoNames { unit: String },     // :name where no member has a name, or not a member unit
    EraModifier,
    NoField,
    UnnameableUnit { name: String },  // a unit name with { } or :
}

/// One date text can be read as.
pub struct Reading { pub date: uni_calendar::Date }

pub enum ReadError {
    Empty,
    WriteOnly(WriteOnly),
    NoMatch { at: usize, expected: String },
    Disagree { unit: String, first: String, second: String },
    Refused(uni_calendar::CalendarError),
}

impl Format {
    pub fn compile(template: &str, calendar: &Calendar) -> Result<Format, PatternError>;
    pub fn generated(calendar: &Calendar) -> Format;                 // R3
    pub fn write_only(&self) -> Option<&WriteOnly>;
    /// The smallest unit the format shows (for the partway mark, FR-016).
    pub fn smallest_shown(&self) -> Option<usize>;
    /// `above` and `date` as `instant_to_date` gives them.
    pub fn write(&self, calendar: &Calendar, found: &InstantDate) -> String;
    pub fn read(&self, calendar: &Calendar, text: &str) -> Result<Vec<Reading>, ReadError>;
}
```

All implement `Display` with messages in the model's terms (ADR 0009), for example `"span" has no member names`, `expected a number for span at character 4`.

## `uni-calendar` (additions)

```rust
impl Calendar {
    /// A member unit's member names, in order; `None` for any other unit.
    pub fn member_names(&self, unit: &str) -> Option<&[Option<String>]>;
    /// How far into the instance of `unit` containing `instant` the moment falls: exact, in [0, 1).
    pub fn fraction_through(&self, instant: Instant, unit: &str) -> R<Ratio>;
}
```

## `uni-store`

```rust
pub struct CalendarRecord {
    // … as spec 008, plus:
    pub formats: Vec<FormatRecord>,
    /// Index into `formats`; `None` means the generated format is the main one.
    pub main_format: Option<usize>,
}
pub struct FormatRecord { pub name: String, pub template: String }
```

- `normalised()` trims format names; templates are kept exactly as typed.
- `SCHEMA_VERSION` = 4; `MIGRATIONS[3]` creates `format` and its index (R7).
- Rows: one per format, `seq` from 0 in list order, `main = 1` on at most one.
- Validity (names unique, patterns compile) is checked by `uni-app` before a record is stored, as the engine's checks are; the table's constraints are the last line.

## `uni-app`

```rust
pub struct FormatForm { pub name: String, pub template: String }
// CalendarForm gains: formats: Vec<FormatForm>, main_format: Option<usize>

pub enum Location { /* … as spec 008, plus: */ Format { index: usize, field: FormatField } }
pub enum FormatField { Name, Template }

/// What check_format gives (R8).
pub enum FormatCheck {
    Sample { text: String, write_only: Option<String> },
    Error { message: String, at: usize },
    Definition { definition: String },
}

pub struct DateReading { pub values: Vec<DateValue>, pub text: String }   // + text
pub struct ConvertedDate {
    pub above: Vec<DateValue>, pub values: Vec<DateValue>,
    pub text: String,              // in the main format
    pub generated: bool,           // the main format is the generated one (show `above` first, R3)
    pub unit: String,              // the smallest unit the main format shows
    pub partway: Option<Partway>,  // measured in `unit`
}
```

## UI

- `CalendarDraft` gains `formats: { name: string; template: string }[]` and `mainFormat: number | null`.
- `FormatsSection` (new): rows with name, pattern, a "main" radio (plus "Generated"), delete; add; each row's live `FormatCheck`.
- `convertedDateText` takes `ConvertedDate` and the precision, and writes `text` with the partway mark naming `unit`.
- `AnchorSection`'s read-back uses `DateReading.text`.

## Lifecycle

- A format exists only inside its calendar's record: added (at the end of the list), edited and deleted in the editor, kept when the calendar is kept, removed with the calendar (cascade). Reordering comes with the full format editor.
- Deleting the main format makes `main_format` `None` (generated); deleting a format before it moves the index down with it.
