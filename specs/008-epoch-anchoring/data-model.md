# Data Model: Epoch Anchoring and the Converter

Terms follow ADR 0009. Spec 007's types (`CalendarRecord`, `StoredCalendar`, `CalendarForm`, `FieldError`, `CalendarSummary`) are unchanged except where this file says so.

## Anchor (`uni-store`)

A calendar's link into time (research R2). The reference calendar has none.

| Field | Type | Rules |
|---|---|---|
| `kind` | `AnchorKind` | `Fixed` (fix the moment, the default) or `Equation` (keep the equation). |
| `to` | `Option<i64>` | The other calendar's id; `None` once it has been deleted. Never this calendar's own id. An `Equation` anchor always has one. |
| `to_name` | `Option<String>` | The other calendar's name, written only when it was deleted (research R6). |
| `date` | `Vec<i64>` | This calendar's date, from the era's unit down; 1 to as many values as there are units from the era's unit down. |
| `to_date` | `Vec<i64>` | The other calendar's date, likewise. |
| `at` | `Option<Instant>` | For `Fixed`: the instant both dates started at when confirmed. `None` for `Equation`, and for a `Fixed` anchor whose follower could not be placed when its calendar was deleted (research R6). Never returned by a command. |

`CalendarRecord` gains `anchor: Option<Anchor>`. `StoredCalendar` is unchanged (it holds the record).

## Placement (`uni-store`)

What `OpenUniverse::placements()` returns for each calendar, in list order (research R4).

| Field | Type | Notes |
|---|---|---|
| `stored` | `StoredCalendar` | As read. |
| `placed` | `Result<Calendar, PlacementProblem>` | The engine's calendar with its epoch worked out, or why it can't be placed. |

`PlacementProblem` (each `Display`s as one sentence in the model's terms):

| Variant | Message (example) |
|---|---|
| `Length(RecordError)` | spec 007's message |
| `Definition(DefinitionError)` | the engine's message |
| `Date { error: CalendarError }` | "its anchoring date: span 13 is outside 1 to 12" |
| `ToDate { name, error: CalendarError }` | "its anchoring date in Aster: span 13 is outside 1 to 12" |
| `ToUnusable { name }` | "it's anchored to Aster, which can't be placed" |
| `Loop { names: Vec<String> }` | "Tallow, Wick and Tallow keep equations with each other in a loop" |
| `NoMoment` | "its anchor has no moment: anchor it again" |
| `OutOfRange` | "its anchor puts it beyond the timeline's range" |

## OpenUniverse changes (`uni-store`)

| Method | Effect |
|---|---|
| `placements() -> Result<Vec<Placement>, StoreError>` | Places every calendar (research R4). |
| `add_calendar`, `replace_calendar` | Also write the record's anchor (or none). Unchanged otherwise. The store doesn't check anchors; `uni-app` does, through `check_anchor`, before calling them (research R5), as spec 007 does for definitions. |
| `check_anchor(id: Option<i64>, record: &CalendarRecord) -> Result<Option<Instant>, AnchorError>` | Research R5 steps 5–9 against the universe as stored. Returns the `at` to store (`Some` for `Fixed`, `None` for `Equation`). |
| `delete_calendar(id)` | Research R6: followers' equations become fixed moments, `to_name` written, then the delete. |
| `followers(id) -> Result<Vec<String>, StoreError>` | Names of the calendars whose anchor keeps an equation with `id`, in list order. |

`AnchorError`: `Missing` (a calendar other than the reference has no anchor), `NotAllowed` (the reference calendar was given one), `ToMissing` (no calendar to equate with, or it doesn't exist), `ToSelf`, `DateEmpty { side }`, `DateGap { side, index }`, `ToUnusable { name, problem }`, `Loop { names }`, `ToDate(CalendarError)`, `Date(CalendarError)`, `OutOfRange`. `side` is `Own` or `To`.

## Engine additions (`uni-calendar`, `uni-time`)

- `Calendar::epoch_for(&self, date: &Date, at: Instant) -> Result<Instant, CalendarError>` (research R1).
- `Calendar::smallest_unit_length(&self) -> Ratio`, exact seconds (research R10).
- `Ratio::to_decimal(self, places: u32) -> (String, bool)`: truncated towards zero, and whether it was exact (research R10).

## Tables (schema version 3)

As in research R3:

| Record part | Rows |
|---|---|
| the anchor, if any | `anchor(calendar_id, kind, to_calendar_id, to_name, date, to_date, at_secs, at_nanos)` |
| no anchor | no `anchor` row: the reference calendar |

`date` and `to_date` are JSON arrays of whole numbers (`[412,3,9]`). Reading an anchor row whose dates aren't arrays of 1 or more whole numbers, or a universe with more than one calendar without an anchor row, is `StoreError::NotAUniverseFile { reason: "it's damaged" }`.

Migration 2 → 3 (research R3): for each calendar after the first by `seq`, `anchor(id, 'fixed', <first id>, NULL, '[1]', '[1]', 0, 0)`.

## CalendarForm changes (`uni-app`, JSON, camelCase)

```text
CalendarForm {
  …spec 007's fields…
  anchor: null                        -- the reference calendar (or the first one added)
        | { kind: "fixed" | "equation",
            to: number | null,          -- null only when the other calendar has been deleted
            date: (number|null)[],      -- trailing nulls are dropped; a null before a number is a gap
            toDate: (number|null)[] }
}
```

`get_calendar` now returns `CalendarDetails`:

```text
CalendarDetails {
  form: CalendarForm
  reference: boolean                  -- this is the reference calendar
  rounded: boolean                    -- the other calendar's date starts between two whole nanoseconds (research R7)
  note: null                          -- research R7, FR-005a
      | { kind: "moved", now: ConvertedDate }   -- the other calendar changed since
      | { kind: "gone", name: string }          -- the other calendar was deleted
}
```

## Location addition (`uni-app`, JSON)

`{ kind: "anchor", field?: "to" | "kind" | "date" | "toDate", index?: number }`. `index` is the value's position in that date, from 0 (research R8).

## Converter shapes (`uni-app`, JSON)

```text
ConverterCalendar { id: number, name: string, units: string[], problem: string | null }
                    -- units from the era's unit down: the fields to fill in

Conversion {
  rounded: boolean                    -- the date's exact start lay between two whole nanoseconds
  results: ConvertedCalendar[]        -- every other placed calendar, in list order
  unusable: { name: string, problem: string }[]
}
ConvertedCalendar { id: number, name: string, date: ConvertedDate }
ConvertedDate {
  above: { unit: string, value: number }[]   -- units above the era's unit
  values: { unit: string, value: number }[]  -- from the era's unit to the smallest unit
  partway: null | { decimal: string, decimalExact: boolean, exact: string }
}

DateError { message: string, index: number | null }   -- index into the entered date
```

## Precision (UI only)

`"partway" | "decimal" | "exact"`, held by `App`; starts at `"partway"` each launch and is never sent to the core or saved (FR-010a).

## State transitions

```text
list ──Converter──▶ converter ──Back to calendars──▶ list
converter: choose calendar / edit date ──▶ convert (all filled fields whole numbers) ──▶ results | DateError
converter: precision switch ──▶ results re-rendered, nothing sent to the core
editor(stored) ──Delete──▶ confirm (naming followers) ──yes──▶ list [followers fixed; universe dirty]
```
