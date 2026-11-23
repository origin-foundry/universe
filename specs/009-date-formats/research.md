# Research: Date Formats

Decisions for spec 009, each with the reason and what else was considered. Names follow ADR 0009; examples use Aster and Tallow (spec 008).

## R1. Pattern syntax: a subset of `/formats/templates.md`

**Decision**: A pattern is literal text and fields in braces. A field is `{unit}` or `{unit:modifier}`:

| Field | Writes | Example (Aster, orbit 412, span 3, sol 9) |
|---|---|---|
| `{orbit}` | the unit's number | `412` |
| `{span:02}` | the number padded with zeros to the width after the `0` (1 to 18 digits) | `03` |
| `{span:name}` | the member's name, or its number when that member has none | `Third` |
| `{era}` | the era's name | `After Landing` |

- `{{` and `}}` write a literal brace. Anything else in braces that isn't a field is refused (R4).
- The text before the last `:` in a field names the unit, matched exactly after trimming spaces, so unit names may contain spaces (`{deep turn:02}`). A unit name containing `{`, `}` or `:` can't be named in a field; the refusal says so.
- `era` names the era only when no unit of the calendar is called `era`; if one is, `{era}` is that unit and the era's name can't be written until eras get their own field syntax with multiple eras.
- Fields may name units above the era's unit (`{cohort}`), the era's unit and every unit below it, in any order, any number of times.

**Rationale**: It's the exact syntax of the long-term design (`{span:name}`, `{span:04}` in `/formats/templates.md`), cut to what this spec needs, so later specs add modifiers (`:abbr`, `:ord`), conditionals and computed values without changing what stored patterns mean. `{era}` stands in for `{era:name}` until eras have abbreviations; later, `{era}` stays the name.

**Alternatives**: a format built from a list of rows in the editor (unit, style, separator) rather than text: easier for a first-time user, but a second representation to migrate when templates grow; strftime-style letters (`%Y`): Earth-shaped and meaningless for owner-defined units.

## R2. Where formats live: `uni-format`, with two small engine accessors

**Decision**: `uni-format` (empty since spec 001) gets the pattern compiler, the writer and the reader. It depends on `uni-calendar` and `uni-time`, edges already allowed by `/architecture/crate-layout.md`. `uni-app` gains a dependency on `uni-format` (also allowed). `uni-store` doesn't need it: it stores patterns as text (R7).

`uni-calendar` gains two public, exact functions:
- `Calendar::member_names(unit) -> Option<&[Option<String>]>`: a member unit's member names, for `:name` fields. Today they're crate-private.
- `Calendar::fraction_through(instant, unit) -> R<Ratio>`: how far into the instance of `unit` containing `instant` the moment falls, as an exact fraction in `[0, 1)`. The converter needs it when a format stops above the smallest unit (FR-016). It's the remainder within that unit's instance divided by the instance's length, both in ticks (the engine's exact grid), so it works for groupings whose members differ in length and for adjustment rules.

**Rationale**: Principle V puts formats in their own crate under the app; principle III makes the crate exact (it already denies float arithmetic). The engine owns unit lengths, so the fraction belongs there.

**Alternatives**: computing the fraction in `uni-app` from two `date_to_instant` calls (start of this instance and of the next): "the next" has no simple form at a grouping's last member, and the rounding to whole nanoseconds would make the fraction inexact.

## R3. The generated format

**Decision**: Built from the calendar's definition as a compiled pattern (not text, so unit names with braces are no problem): for each unit from the era's unit down, `<unit> {<unit>}`, joined by `, `; after a member unit's number, ` ({<unit>:name})` when every member of it has a name. Aster: `orbit 412, span 3 (Third), sol 9, bell 0, pulse 0`. It's readable (R5), so later text input can always fall back on it.

- Units above the era's unit aren't in it: they're derived, and today's converter shows them separately. The converter keeps showing them before the formatted text, as now ("age 1, cohort 42, …") only when the main format is the generated one; an owner's format shows exactly what it says.
- **Difference from today's text, for the owner to confirm in design review**: today a grouping where only some members are named shows names only on those members. The generated format shows names only when all are named, because a pattern can't yet say "a name in parentheses if there is one" (that needs conditionals). Aster and Tallow are unaffected: SC-004 holds for calendars whose member units are wholly named or wholly unnamed.

**Alternatives**: a special-case writer that reproduces today's text exactly: not a pattern, so it couldn't also read text, and it would be a second writer to keep in step.

## R4. Checking a pattern (FR-003)

**Decision**: `Format::compile(pattern, &Calendar)` returns the compiled format or a `PatternError` with the character position (from 0, in characters) and the reason. Checks, first fault wins, left to right:

1. An unclosed `{`, or a `}` not doubled and not closing a field.
2. An empty field `{}`, or a unit name that isn't one of the calendar's units (nor `era`).
3. An unknown modifier, or a padding width outside 1 to 18.
4. `:name` on a unit that isn't a member unit, or whose members have no names at all.
5. A modifier on `{era}`.
6. No field at all.

A format that compiles is then **readable** or **write-only** (R5). The format's name is checked by `uni-app` with the form: not empty after trimming, unique in the calendar (case-insensitively, as calendar names are).

## R5. Readable or write-only (FR-004)

**Decision**: Take the units the fields name, ignoring units above the era's unit and `{era}`. The format is readable when those units are exactly a run from the era's unit down (the era's unit, then optionally the next, and so on, with no gap). Otherwise it's write-only, with the reason: "it doesn't show the orbit, so text in it doesn't say which orbit" or "it shows orbit and sol but not span, so text in it doesn't say which span". A pattern stopping early is readable; it reads partial dates.

## R6. Reading text (FR-009 to FR-013)

**Decision**: `Format::read(text, &Calendar) -> Result<Vec<Reading>, ReadError>`, a backtracking match of the compiled pattern against the text.

- **Literal text** matches character by character after simple lowercasing of both sides; a run of whitespace in the pattern matches one or more whitespace characters, and text whitespace must be matched by pattern whitespace. Leading and trailing whitespace in the text is ignored.
- **Number fields** match an optional `-` (also `−`, U+2212) and one or more ASCII digits. Where the field is followed directly by another number or name field (no literal between), every split is tried; otherwise the field takes the longest run. A padded field touching another field with no literal between takes exactly its width in digits; elsewhere it takes any number of digits, so `412.3.9` reads in `{orbit}.{span:02}.{sol:02}`.
- **Name fields** match any member name of that unit, case-insensitively, longest first, or a number (for members with or without names).
- **`{era}`** matches the era's name case-insensitively.
- Each complete match gives a candidate: the values for the readable run, from the era's unit down. When a unit appears twice, both must agree; when a unit above the era's unit appears, it must equal what the engine works out (`instant_to_date` of the date's start). A candidate failing these gives a disagreement, naming the unit and both values.
- Candidates are checked by `Calendar::date_to_instant`; refused ones are dropped. Duplicates (the same date by two routes) are merged.
- Results: one or more valid readings, in the order found; or, with none, a `ReadError`: `NoMatch { at, expected }` (the furthest position any attempt reached and what the pattern wanted there: "a number for span", "\".\"", "a name of a span member: Prime, Second, …", "the end of the text"), `Disagree { unit, first, second }`, `Refused(CalendarError)` (the first candidate's), or `WriteOnly` for a write-only format. Empty text (after trimming) is `Empty`.
- Number text that doesn't fit an `i64` is a `NoMatch` at that number ("a number too large").

**Rationale**: Patterns have at most a handful of fields, so backtracking stays small (bounded by the text's digit runs); principle VII is met by returning every reading rather than choosing.

## R7. Storage: schema version 4 (FR-018)

**Decision**: Migration 3 → 4 adds:

```
format(id INTEGER PRIMARY KEY,
       calendar_id INTEGER NOT NULL REFERENCES calendar (id) ON DELETE CASCADE,
       seq INTEGER NOT NULL,            -- order in the calendar's list
       name TEXT NOT NULL,              -- trimmed, unique in the calendar
       template TEXT NOT NULL,          -- the pattern exactly as typed
       main INTEGER NOT NULL DEFAULT 0 CHECK (main IN (0, 1)),
       UNIQUE (calendar_id, seq), UNIQUE (calendar_id, name))
CREATE UNIQUE INDEX format_one_main ON format (calendar_id) WHERE main = 1;
```

No row is `main` → the generated format is the main one. `CalendarRecord` gains `formats: Vec<FormatRecord { name, template }>` and `main_format: Option<usize>`, written and read with the calendar's other rows. Migrated files get no rows (FR-018). `kind` and `inverse_overrides_json` of the target schema come with duration formats and computed values; `locale` with locales.

**Rationale**: Columns match the target design's `format` table where they overlap (`template`), so later specs add columns rather than reshape. The partial unique index enforces "one main format" in the file.

## R8. Checking formats in the editor (FR-003)

**Decision**: A new command `check_format(form, template) -> FormatCheck` compiles one pattern against the editor's draft (as `read_date` does for the anchor) and returns the sample text and whether it's write-only (with the reason), or the pattern error with its position, or that the draft's definition isn't valid yet. The UI calls it as each pattern changes, debounced as `read_date` is. On keep, `save_calendar`/`add_calendar` compile every format again; a failure is located at `Location::Format { index, field }` (name or pattern), using spec 007's location mechanism, and the calendar isn't kept.

**Sample date**: the draft's anchor date when it has one that the draft accepts, else its first date (every value at its first number), written with `Calendar::instant_to_date` of its start so units above the era's unit are filled in.

## R9. A stored format that no longer compiles

**Decision**: Can only happen if the file was changed outside the app or a later app tightens a rule. Where the app writes dates (converter, read-back), a calendar whose main format doesn't compile uses its generated format; opening it in the editor shows the format's error at the format, and keeping is refused until it's fixed (as spec 007 does for a stored definition the engine refuses).

## R10. Converter results and the read-back sentence (FR-015 to FR-017)

**Decision**:
- `ConvertedDate` keeps `above`, `values` and gains `text` (the date in the main format) and `unit` (the smallest unit the main format shows). `partway` is measured in that unit with `Calendar::fraction_through` and keeps its three forms. The UI writes `text`, then, when partway: "partway through the sol" / "≈ 0.370 of the way through the sol" (or "0.375 of the way…" when exact) / "37/100 of the way through the sol". With the generated format, `above` is shown before `text`, as now (R3).
- `DateReading` gains `text`: the full date in the calendar's main format (for a draft, the draft's main format). The read-back sentence becomes "Tallow's {text} began at the start of Aster's {text}." It shows the full date, since a date stopping early starts at its smaller units' first values anyway; spec 008's "only the values entered" build choice is replaced.

## R11. Tests and timing

- `uni-format`: unit tests for each refusal (R4), each read error (R6), padding and negative values, member names, `{era}`, units above the era's unit; property tests (proptest, at least 1,000 cases each, SC-002) over Aster, Tallow and a Gregorian-shaped test calendar's generated formats and a fixed set of owner-style patterns (padded, named, separator-free padded, with `{cohort}`, repeated unit): write then read gives exactly the original date as the only reading. Separator-free unpadded patterns are tested separately: the original is always among the readings.
- `uni-calendar`: `fraction_through` against hand-worked values, and a property test that it equals the smallest-unit remainder over the smallest unit's length for the smallest unit.
- `uni-store`: the migration (backup made, no rows), round trip of formats and main choice, the one-main index.
- `uni-app`: `check_format` outcomes, refusal locations on keep, converter text and partway unit, read-back text.
- UI: the formats section, the converter's text, the read-back sentence.
- Timing (SC-005): write and read one date in a calendar with a 1,000-member grouping, release build, in a test like spec 008's T024, under 10 ms each.

## R12. Size (principle VIII)

**Estimate**: about 2,800 changed lines: `uni-format` 1,000 (half tests), `uni-calendar` 150, `uni-store` 250, `uni-app` 450, UI 600, specs and knowledge the rest. That is below spec 008's build (about 3,300), so one build PR. If the build runs long, the converter and read-back changes (US3) can move to their own build PR after formats and their editing (US1, US2): tasks keep them in a separate phase.
