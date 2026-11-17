# Data Model: Calendars in the Project File

Terms follow ADR 0009. Engine types (`CalendarDef`, `GroupingDef`, …) are spec 004's and spec 006's, unchanged.

## CalendarRecord (`uni-store`)

A calendar as the owner entered it (research R1).

| Field | Type | Rules |
|---|---|---|
| `name` | `String` | Trimmed; 1–200 characters; unique in the universe (research R4). |
| `groupings` | `Vec<GroupingDef>` | As `CalendarDef.groupings`, top to bottom; may be empty. |
| `fundamental` | `FundamentalRecord` | Exactly one. |
| `subdivisions` | `Vec<SubdivisionDef>` | As `CalendarDef.subdivisions`; may be empty. |
| `era` | `EraRecord` | Exactly one. |

`FundamentalRecord { name: String, length: String, first_number: Option<i64> }`. `length` is the trimmed text typed, parsed by `Ratio::parse` (research R3).

`EraRecord { name: String, unit: String }`. There's no epoch: `to_def()` sets `Instant::ZERO` (research R6).

Groupings, members, subdivisions and adjustment rules reuse the engine's `GroupingDef`, `MemberUnitDef`, `MemberDef`, `SubdivisionDef` and `AdjustmentRuleDef`, whose fields are already exactly what's entered. A member name that is empty after trimming is stored as `None`.

**Conversion**: `CalendarRecord::to_def(&self) -> Result<CalendarDef, RecordError>`, where `RecordError` is `Length { text, error: TimeError }`. `Calendar::new(def)` does the rest of the checking.

## StoredCalendar (`uni-store`)

What reading a universe returns for each calendar.

| Field | Type | Notes |
|---|---|---|
| `id` | `i64` | Stable for the calendar's life, across edits (spec 008 will anchor to it). |
| `record` | `CalendarRecord` | Exactly as stored. |

The list is ordered by `calendar.seq`. A new calendar goes at the end; deleting one doesn't renumber the others (order is what's compared, not the numbers).

## OpenUniverse additions (`uni-store`)

| Method | Effect |
|---|---|
| `calendars() -> Result<Vec<StoredCalendar>, StoreError>` | Reads all calendars in order. |
| `calendar(id) -> Result<StoredCalendar, StoreError>` | Reads one; `StoreError::NoSuchCalendar` if absent. |
| `add_calendar(&CalendarRecord) -> Result<i64, StoreError>` | Inserts at the end; `StoreError::CalendarNameTaken` or `CalendarName` (empty or too long) on a bad name. Marks dirty. |
| `replace_calendar(id, &CalendarRecord) -> Result<(), StoreError>` | Replaces all its rows in one transaction, keeping `id` and position. The same name checks, except against itself. Marks dirty. |
| `delete_calendar(id) -> Result<(), StoreError>` | Deletes it and, by cascade, its rows. Marks dirty. |

The store checks names but not definitions: `uni-app` checks the definition with the engine before calling `add_calendar` or `replace_calendar` (research R8). The store stays a faithful keeper of records, and an invalid record can still be read back (research R9).

## Tables (schema version 2)

As in research R2. Mapping:

| Record part | Rows |
|---|---|
| `CalendarRecord.name`, position | `calendar(id, seq, name)` |
| each grouping, in order | `unit(kind='grouping', seq, name, first_number, member_unit_name, member_unit_first_number)` |
| the fundamental unit | `unit(kind='fundamental', seq, name, first_number, length)` |
| each subdivision, in order | `unit(kind='subdivision', seq, name, first_number, parts)` |
| each member | `unit_member(unit_id, seq, name, count)` |
| each adjustment rule | `rule(unit_id, seq, condition_src, member, change)` on its grouping's row |
| the era | `era(calendar_id, seq = 1, name, unit_name)` |

The CHECK constraints are:
- `length` is set exactly when `kind='fundamental'`;
- `parts` is set exactly when `kind='subdivision'`;
- `member_unit_*` is set only when `kind='grouping'`.

Reading rows that break the expected shape (no fundamental unit, two eras, units out of the grouping–fundamental–subdivision order) is `StoreError::NotAUniverseFile { reason: "it's damaged" }`, as spec 003 does for a damaged file.

## CalendarForm (`uni-app`, JSON, camelCase)

The UI's mirror of `CalendarRecord`. Whole numbers are `number | null`, and a required one that's `null` is reported missing (research R5).

```text
CalendarForm {
  name: string
  groupings: [{ name, firstNumber: number|null,
                memberUnit: { name, firstNumber: number|null } | null,
                members: [{ name: string|null, count: number|null }],
                rules: [{ condition, member, change: number|null }] }]
  fundamental: { name, length: string, firstNumber: number|null }
  subdivisions: [{ name, parts: number|null, firstNumber: number|null }]
  era: { name, unit }
}
```

## FieldError (`uni-app`, JSON)

`{ message: string, at: Location }`. `Location` is one of:
- `{ kind: "calendarName" }`
- `{ kind: "era", field: "name" | "unit" }`
- `{ kind: "grouping", index, field?: "name" | "firstNumber" | "memberUnit" }`
- `{ kind: "member", grouping, index, field?: "name" | "count" }`
- `{ kind: "rule", grouping, index, field?: "condition" | "member" | "change" }`
- `{ kind: "fundamental", field?: "name" | "length" | "firstNumber" }`
- `{ kind: "subdivision", index, field?: "name" | "parts" | "firstNumber" }`

Indices count from 0 in the form. The UI shows the message next to the named input, or at the top of the section when `field` is absent.

## CalendarSummary (`uni-app`, JSON)

`{ id: number, name: string, problem: string | null }`. `problem` is the engine's (or length parser's) message for a stored calendar that's refused (FR-015).

## State transitions

```text
list ──Add calendar──▶ editor(new draft) ──Keep (valid)──▶ list  [universe dirty]
list ──choose──▶ editor(stored draft) ──Keep (valid)──▶ list  [universe dirty]
editor ──Keep (invalid)──▶ editor + FieldError   [nothing stored]
editor ──Discard──▶ list   [nothing stored]
editor(stored) ──Delete, confirmed──▶ list  [universe dirty]
editor with changes ──New/Open/Close/Exit──▶ ask keep or discard ──▶ existing unsaved-universe flow
```
