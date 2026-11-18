# Contract: anchors and the converter in `uni-time`, `uni-calendar`, `uni-store` and the app's commands

Additions and changes only; spec 007's contract (`specs/007-calendars-in-project/contracts/calendar-commands.md`) holds otherwise.

## `uni-time` (Rust)

```rust
impl Ratio {
    /// The value to `places` decimal places, truncated towards zero, and whether that's exact.
    /// `Ratio(37, 100).to_decimal(3) == ("0.370", true)`; `Ratio(1, 3).to_decimal(3) == ("0.333", false)`.
    pub fn to_decimal(self, places: u32) -> (String, bool);
}
```

## `uni-calendar` (Rust)

```rust
impl Calendar {
    /// The epoch that makes `date` start at `at`: with it, `date_to_instant(date).instant == at`.
    /// Errors as `date_to_instant`, or `OutOfRange` if the epoch would leave the timeline.
    pub fn epoch_for(&self, date: &Date, at: Instant) -> Result<Instant, CalendarError>;

    /// The smallest unit's length in SI seconds, exact: the fundamental unit's length divided by
    /// every subdivision's parts.
    pub fn smallest_unit_length(&self) -> Ratio;
}
```

## `uni-store` (Rust)

```rust
pub const SCHEMA_VERSION: u32 = 3;     // MIGRATIONS gains 2 → 3 (research R3)

pub struct CalendarRecord { /* spec 007's fields, plus: */ pub anchor: Option<Anchor> }

pub enum AnchorKind { Fixed, Equation }
pub struct Anchor {
    pub kind: AnchorKind,
    pub to: Option<i64>,
    pub to_name: Option<String>,
    pub date: Vec<i64>,
    pub to_date: Vec<i64>,
    pub at: Option<uni_time::Instant>,
}

pub struct Placement { pub stored: StoredCalendar, pub placed: Result<uni_calendar::Calendar, PlacementProblem> }
pub enum PlacementProblem {
    Length(RecordError), Definition(uni_calendar::DefinitionError),
    Date { error: uni_calendar::CalendarError },
    ToDate { name: String, error: uni_calendar::CalendarError },
    ToUnusable { name: String }, Loop { names: Vec<String> }, NoMoment, OutOfRange,
}

pub enum AnchorSide { Own, To }
pub enum AnchorError {
    Missing, NotAllowed, ToMissing, ToSelf,
    DateEmpty { side: AnchorSide }, DateGap { side: AnchorSide, index: usize },
    ToUnusable { name: String, problem: PlacementProblem },
    Loop { names: Vec<String> },
    ToDate(uni_calendar::CalendarError), Date(uni_calendar::CalendarError), OutOfRange,
}

impl OpenUniverse {
    pub fn placements(&self) -> Result<Vec<Placement>, StoreError>;
    pub fn check_anchor(&self, id: Option<i64>, record: &CalendarRecord) -> Result<Option<Instant>, AnchorError>;
    pub fn followers(&self, id: i64) -> Result<Vec<String>, StoreError>;
    // add_calendar and replace_calendar store record.anchor; delete_calendar follows research R6.
}
```

Messages (`Display`), in the model's terms, never naming seconds or the timeline's values:

| `AnchorError` | Message |
|---|---|
| `Missing` | "Choose a calendar to equate this one with." |
| `NotAllowed` | "The reference calendar isn't anchored: other calendars are placed from it." |
| `ToMissing` | "Choose a calendar to equate this one with." |
| `ToSelf` | "A calendar can't be equated with itself." |
| `DateEmpty` | "Enter at least the <era's unit>." |
| `DateGap` | "Fill in <unit>, or leave every smaller unit empty." |
| `ToUnusable` | "<name> can't be used: <problem>" |
| `Loop` | "<A>, <B> and <A> would keep equations with each other in a loop." |
| `ToDate(e)`, `Date(e)` | the engine's message |
| `OutOfRange` | "This anchor would put the calendar beyond the timeline's range." |

## Tauri commands (`uni-app`)

| Command | Arguments | Returns | Errors |
|---|---|---|---|
| `list_calendars` | none | `CalendarSummary[]`; `problem` now includes placement problems | string |
| `get_calendar` | `id` | `CalendarDetails` (was `CalendarForm`) | string |
| `new_calendar_form` | none | `CalendarForm` with its anchor (research R7) | string |
| `put_calendar` | `id \| null`, `form` | unchanged | `FieldError`, now also at `anchor` (research R8), or string |
| `delete_calendar` | `id` | unchanged; research R6 | string |
| `calendar_followers` | `id` | `string[]` (new) | string |
| `converter_calendars` | none | `ConverterCalendar[]` (new) | string |
| `convert` | `from: number`, `date: (number \| null)[]` | `Conversion` (new) | `DateError`, or string for a store failure or an unusable source calendar |

Every command needs an open universe, except `new_calendar_form` as before; with none, it returns "No universe is open." Shapes are in [data-model.md](../data-model.md).

## Where each anchor refusal is placed

| Refusal | Location |
|---|---|
| `Missing`, `ToMissing`, `ToSelf`, `ToUnusable` | `{ kind: "anchor", field: "to" }` |
| `NotAllowed` | `{ kind: "anchor" }` |
| `DateEmpty { side }` | `field: "date"` or `"toDate"`, `index: 0` |
| `DateGap { side, index }` | that side's field, that `index` |
| `Loop` | `{ kind: "anchor", field: "kind" }` |
| `ToDate(InvalidValue { unit, .. })`, `Date(InvalidValue { unit, .. })` | that side's field, `index` = the unit's position from the era's unit |
| `ToDate`/`Date` with any other `CalendarError`, `OutOfRange` | that side's field (or `{ kind: "anchor" }` for `OutOfRange`), no `index` |

The same `CalendarError` placement gives `DateError.index` in `convert`. The match over `CalendarError` is total, with no `_` arm.
