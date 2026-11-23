# Contract: date formats in `uni-format`, `uni-calendar`, `uni-store` and the app's commands

Additions and changes only; spec 008's contract (`specs/008-epoch-anchoring/contracts/anchors-and-converter.md`) holds otherwise. Shapes are in [../data-model.md](../data-model.md).

## `uni-format` (Rust)

`Cargo.toml` gains `uni-calendar` and `uni-time` (path dependencies, `publish = false`), and `proptest` as a dev-dependency.

| Call | Gives | Errors |
|---|---|---|
| `Format::compile(template, &calendar)` | a `Format` | `PatternError { at, kind }` (research R4) |
| `Format::generated(&calendar)` | the generated format (R3), always readable | none |
| `format.write_only()` | `Some(&WriteOnly)` with the reason, or `None` | — |
| `format.smallest_shown()` | the index (in `calendar.units()`) of the smallest unit a field shows, or `None` for a pattern of only `{era}` and units above the era's unit | — |
| `format.write(&calendar, &found)` | the text | none: every field has a value in an `InstantDate` |
| `format.read(&calendar, text)` | one or more `Reading`s, in the order found, no duplicates | `ReadError` (R6) |

**Guarantees**

- `write` then `read` of any date of the calendar, in a readable format with no two number or name fields touching, gives exactly one reading: that date, cut to the units the format shows (SC-002).
- With touching fields, the original date is always among the readings.
- `read` never returns a date the calendar refuses.
- No floating point (the crate denies it).

**Example (Aster)**

```rust
let short = Format::compile("{orbit}.{span:02}.{sol:02}", &aster)?;
assert_eq!(short.write(&aster, &aster.instant_to_date(start_of(412, 3, 9))?), "412.03.09");
assert_eq!(short.read(&aster, "412.3.9")?, vec![Reading { date: Date { values: vec![412, 3, 9] } }]);
let err = short.read(&aster, "412-03-09").unwrap_err();
assert_eq!(err.to_string(), "expected \".\" at character 3");
```

## `uni-calendar` (Rust)

```rust
impl Calendar {
    pub fn member_names(&self, unit: &str) -> Option<&[Option<String>]>;
    pub fn fraction_through(&self, instant: Instant, unit: &str) -> Result<Ratio, CalendarError>;
}
```

`fraction_through` gives `UnknownUnit` for a unit the calendar doesn't have and `OutOfRange` as `instant_to_date` does. For the smallest unit it equals `instant_to_date(instant).remainder / smallest_unit_length()`.

## `uni-store` (Rust)

```rust
pub const SCHEMA_VERSION: u32 = 4;     // MIGRATIONS gains 3 → 4 (research R7)
pub struct CalendarRecord { /* spec 008's fields, plus: */ pub formats: Vec<FormatRecord>, pub main_format: Option<usize> }
pub struct FormatRecord { pub name: String, pub template: String }
```

`add_calendar`, `replace_calendar` and `calendar(s)` write and read the formats with the rest of the record; a version-3 file migrates with its backup (`<name>.v3.universe`) and no formats.

## App commands (Tauri)

| Command | Change |
|---|---|
| `check_format(source: DateSource, template: string) -> FormatCheck` | **New** (R8). `source` is a stored calendar or the editor's draft, as for `read_date`. |
| `get_calendar`, `new_calendar_form` | The form gains `formats: {name, template}[]` and `mainFormat: number \| null`. |
| `put_calendar` | Checks every format (name not empty, unique case-insensitively; pattern compiles); a refusal is located at `{ format: { index, field: "name" \| "template" } }`, with the message. |
| `read_date` | `DateReading` gains `text`: the full date in the calendar's main format. |
| `convert` | Each `ConvertedDate` gains `text`, `generated` and `unit`; `partway` is measured in `unit` (R10). |

`FormatCheck` on the wire:

```json
{ "sample": "9 Third, orbit 412 After Landing", "writeOnly": null }
{ "sample": "9 Third", "writeOnly": "It doesn't show the orbit, so text in it doesn't say which orbit." }
{ "message": "\"tide\" isn't a unit of this calendar", "at": 1 }
{ "definition": "…" }
```

## Messages (ADR 0009 terms)

| Case | Message |
|---|---|
| Unclosed | `this field isn't closed with "}"` |
| Stray brace | `write "}}" for a "}" in the text` |
| Empty field | `this field doesn't name a unit` |
| Unknown unit | `"tide" isn't a unit of this calendar` |
| Unknown modifier | `":abbr" isn't a way to write a value; use a number like ":02" or ":name"` |
| Bad width | `the width must be 1 to 18 digits` |
| No names | `"sol" has no member names` |
| Era modifier | `the era's name is written as "{era}"` |
| No field | `a format needs at least one field, such as "{orbit}"` |
| Unnameable unit | `"a:b" can't be named in a field because of its ":"` |
| Write-only, missing | `It doesn't show the orbit, so text in it doesn't say which orbit.` |
| Write-only, gap | `It shows orbit and sol but not span, so text in it doesn't say which span.` |
| Duplicate name | `there's already a format called "Short"` |
| Empty name | `a format needs a name` |
