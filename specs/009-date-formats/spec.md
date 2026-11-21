# Feature Specification: Date Formats

**Feature Branch**: `design/009-date-formats`

**Created**: 2026-11-21

**Status**: Draft

**Input**: User description: "Date formats (issue #61, Phase 1. Calendar core). Owner-defined patterns for writing a calendar's date as text: which units appear, in what order, separators and literal text, each value as a number (optionally zero-padded) or as its member name; the same pattern parses text back into a date, with unreadable or ambiguous text reported with the reason (principle VII). Formats are saved with their calendar. Used by the converter's results and input and by the anchor's read-back sentence (spec 008). Keep it to one build PR; locales, conditionals, computed values, zone tokens, ordinals/roman/words, duration formats and the full format editor come later. Earth-neutral names (ADR 0009)."

**Issue**: #61 · **Depends on**: spec 004 (calendar engine basics), spec 007 (calendars in the project file), spec 008 (epoch anchoring and the converter) · **Phase**: 1. Calendar core · **Knowledge**: [/formats/templates.md](../../knowledge/formats/templates.md), [/formats/parsing.md](../../knowledge/formats/parsing.md), [/calendar/unit-kinds.md](../../knowledge/calendar/unit-kinds.md), [/calendar/calendar-model.md](../../knowledge/calendar/calendar-model.md), [/calendar/eras-and-epoch.md](../../knowledge/calendar/eras-and-epoch.md), [/storage/schema.md](../../knowledge/storage/schema.md), [/product/ui-views.md](../../knowledge/product/ui-views.md), [ADR 0009](../../knowledge/decisions/0009-earth-neutral-names.md)

## User Scenarios & Testing *(mandatory)*

The examples use the test calendars of spec 008. **Aster**: an age of 10 cohorts of 10 orbits; an orbit of twelve spans (the third is named Third, and so on) counted in sols; a sol of 20 bells of 100 pulses; bells and pulses numbered from 0. **Tallow**: a cycle of 52 rounds of 7 watches, with no member names.

A **format** is a pattern of literal text and **fields**. A field names one of the calendar's units, or its era, and says how its value is written. In this spec's examples a field is written in braces: `{orbit}` is the orbit's number, `{span:02}` the span's number padded to two digits, `{span:name}` the span's member name, `{era}` the era's name. The exact syntax is settled in planning; these examples show what it must be able to say.

### User Story 1 - Define a format and see dates in it (Priority: P1)

In Aster's editor the owner adds a format called "Short", with the pattern `{orbit}.{span:02}.{sol:02}`, and another called "Long", with `{sol} {span:name}, orbit {orbit} {era}`. As they type each pattern, the editor shows a sample date written in it ("412.03.09"; "9 Third, orbit 412 After Landing"), or what's wrong with the pattern. They pick "Long" as Aster's main format, keep the calendar, save, reopen, and the formats are there as entered.

**Why this priority**: Writing dates the owner's way is the point of the feature, and the other stories use the formats defined here.

**Independent Test**: Add the two formats to Aster, check the samples, save and reopen, and check both formats and the main-format choice come back unchanged.

**Acceptance Scenarios**:

1. **Given** Aster in the editor, **When** the owner adds a format with a valid pattern, **Then** a sample date is shown written in it as they type.
2. **Given** a pattern that names a unit the calendar doesn't have, asks for member names of a unit whose members have none, has an unclosed field, or has no field at all, **When** the owner types it, **Then** the problem is shown at the format with the reason, and keeping the calendar is refused until it's fixed.
3. **Given** Aster with formats, **When** the owner saves, closes and reopens the universe, **Then** the same formats, in the same order, with the same main format, are there.
4. **Given** a calendar the owner has given no formats, **Then** it still has a main format: one generated from its definition that reads like the dates the app shows today ("orbit 412, span 3 (Third), sol 9, bell 0, pulse 0"), so nothing has to be set up before dates can be shown.
5. **Given** a pattern that includes a unit above the era's unit (`{cohort}`), **Then** it is accepted and the sample shows that unit's number, worked out from the date.

---

### User Story 2 - Type a date in a format (Priority: P1)

In the converter, the owner picks Aster and one of its formats, and types `412.03.09`. The text is read back into a date in Aster and converted, exactly as if they had entered orbit 412, span 3, sol 9 value by value. Text that can't be read, or that names a date the calendar refuses, is reported with the reason; text that could be read as more than one date offers each reading for the owner to choose.

**Why this priority**: A format that can only write dates is half a format: the knowledge bundle requires every format to parse what it writes (`/formats/parsing.md`) and the constitution requires a round-trip test of every format (principle VI).

**Independent Test**: For each of Aster's formats, write a range of dates in it and read each back; every one gives the original date. Then type malformed and ambiguous text and check the reports.

**Acceptance Scenarios**:

1. **Given** Aster's "Short" format, **When** the owner types `412.03.09` or `412.3.9`, **Then** it is read as orbit 412, span 3, sol 9.
2. **Given** Aster's "Long" format, **When** the owner types `9 third, orbit 412 after landing`, **Then** the member name and era name are matched regardless of capital letters, and the date is read as orbit 412, span 3, sol 9.
3. **Given** text that doesn't fit the pattern (`412-03-09` for "Short", or an unknown member name), **When** it is read, **Then** the owner is told where the text stops fitting and what was expected there, and no date is taken.
4. **Given** text that fits the pattern but names a date the calendar refuses (`412.13.01`, span 13), **When** it is read, **Then** the engine's reason is shown, as for a date entered value by value.
5. **Given** a pattern whose fields touch with no separator, such as `{orbit}{span}`, **When** the owner types `41211`, **Then** both valid readings are offered (orbit 4121, span 1; orbit 412, span 11) and neither is picked silently; readings the calendar refuses (orbit 41, span 211) are not offered; and when there is exactly one valid reading (`4123`: only orbit 412, span 3), it is taken.
6. **Given** a pattern that includes `{cohort}`, **When** the text's cohort doesn't match the cohort the orbit falls in, **Then** the mismatch is reported, not ignored.
7. **Given** a date before the epoch, **When** it is written and read back, **Then** its negative or zero values (astronomical numbering) survive the round trip, padded after the sign (`-007`).

---

### User Story 3 - Formats where the app shows dates (Priority: P2)

The converter shows each result in its calendar's main format, and the anchor's read-back sentence writes both dates in their calendars' main formats: "Tallow's 1/1/1 began at the start of Aster's 9 Third, orbit 412 After Landing."

**Why this priority**: It is what makes formats visible in everyday use; the converter and the read-back sentence are the two places the app writes dates today.

**Independent Test**: Give Aster a main format, convert a Tallow date into Aster and check the result is written in that format; open Tallow's anchor and check the sentence uses both main formats.

**Acceptance Scenarios**:

1. **Given** Aster's main format is "Long", **When** the converter shows a result in Aster, **Then** it is written in "Long".
2. **Given** a result partway through the smallest unit the format shows, **When** the converter's precision is partway, decimal or exact (spec 008, FR-010), **Then** the date is written in the format and the precision's mark follows it, naming that unit ("…, 0.370 of the way through the sol"), never in seconds.
3. **Given** Tallow anchored to Aster, **When** the owner opens Tallow's anchor, **Then** the read-back sentence writes the anchor date in Tallow's main format and the base date in Aster's.
4. **Given** the converter, **When** the owner switches the result format for a target calendar to another of its formats, **Then** that result is rewritten in it; the choice lasts while the app is open, like the precision switch.

---

### Edge Cases

- **Fields left out**: a pattern may leave out units. A pattern that leaves out the era's unit, or a unit between two it includes, can still write dates but can't read them back unambiguously; such a format is marked **write-only** with the reason, offered for results but not for input.
- **Patterns that stop early**: a pattern may stop above the smallest unit (`{orbit}.{span}`); text read in it gives a partial date, whose missing smaller units take their first value, as the engine already does. In results, the precision mark then refers to the last unit the format shows.
- **Member names and numbers together**: a pattern may show the same unit twice (`{span} ({span:name})`); when reading, the two must agree, or the mismatch is reported.
- **Members without names**: in a grouping where only some members are named, `{span:name}` writes the number for an unnamed member and reads either.
- **Calendar edits**: renaming or removing a unit or member that a format uses makes that format invalid with the reason, and keeping the calendar is refused until the format is fixed (the calendar's other checks work the same way, spec 007). The generated format follows the definition by itself.
- **Literal text and spacing**: literal text matches regardless of capital letters; any run of spaces in the pattern matches any run of spaces in the text. A literal brace is written by doubling it.
- **Numbers**: values are whole numbers, possibly negative or zero. A padded field writes at least its width in digits, after any minus sign; it reads with or without its leading zeros, except where it touches another number with no separator, when it reads exactly its width.
- **Empty or over-long text**: empty text reads as no date; text longer than the pattern can consume is reported at the first character that doesn't fit.
- **Earth terms**: formats contain whatever words the owner types; the app's own labels and messages use the calendar model's terms (ADR 0009).

## Requirements *(mandatory)*

### Functional Requirements

**Patterns**

- **FR-001**: A calendar MUST be able to have any number of named, ordered formats. Format names MUST be unique within the calendar and not empty.
- **FR-002**: A format's pattern MUST be literal text and fields. A field MUST name a unit of the calendar (from the units above the era's unit down to the smallest unit) or the era, and say how the value is written:
  - a unit's **number**, as is or **padded** with leading zeros to a given width;
  - a unit's **member name**, for a member unit (or a grouping's single member level) whose members have names;
  - the era's **name**.
- **FR-003**: A pattern MUST be checked as it is typed, and MUST be refused, with the reason shown at the format, when it has no field, names a unit or modifier that doesn't exist, asks for member names where no member has one, or has an unclosed or stray brace. Keeping the calendar MUST be refused while any of its formats is refused.
- **FR-004**: A format MUST be marked **write-only**, with the reason, when it can't be read back into a unique date for every date it writes: when it leaves out the era's unit, or a unit between two units it shows. Write-only formats MUST still write dates.
- **FR-005**: Each calendar MUST have exactly one **main format**. If the owner has defined none, or chooses none, the main format MUST be one **generated** from the definition that writes each unit's name and number from the era's unit down, with member names in parentheses where the member unit's members are all named ("orbit 412, span 3 (Third), sol 9, bell 0, pulse 0"). The generated format is not stored; it follows the definition, and can't be edited.

**Writing dates**

- **FR-006**: Writing a date in a format MUST give the pattern's literal text with each field replaced by its value. Units above the era's unit MUST be written as the numbers the engine works out from the date.
- **FR-007**: A padded number MUST be written with at least its width in digits, after a minus sign when negative, and never cut short.
- **FR-008**: A member-name field MUST write the member's name, or its number when that member has no name.

**Reading text**

- **FR-009**: Reading text in a format that isn't write-only MUST give every date the text can be read as, using the same pattern. Member names, the era's name and literal text MUST match regardless of capital letters; a run of spaces MUST match any run of spaces.
- **FR-010**: Text that can't be read MUST be reported with where it stops fitting the pattern and what was expected there. No date MUST be taken from it.
- **FR-011**: Text that can be read as more than one valid date MUST offer every valid reading for the owner to choose; one MUST NOT be picked silently. Readings the calendar refuses MUST be dropped; if none is valid, the engine's reason for the first MUST be shown.
- **FR-012**: When the text gives a value twice (a number and a member name for the same unit), or gives a unit above the era's unit, the values MUST agree with the date read; a disagreement MUST be reported with both values.
- **FR-013**: For every format that isn't write-only, reading back what it writes MUST give the original date, for every date of the calendar (principle VI's round trip).

**Where formats are used**

- **FR-014**: The converter MUST take its input as text in a format of the source calendar, chosen from that calendar's readable formats (the main format by default if readable, otherwise the generated one), checked as the owner types.
- **FR-015**: The converter MUST write each result in its calendar's main format by default, and let the owner switch each target calendar's result to another of its formats; the switch MUST last while the app is open and not be saved in the universe file.
- **FR-016**: In a result partway through a unit, the precision mark (spec 008, FR-010) MUST follow the formatted date and name the smallest unit the format shows; how far into that unit the moment falls MUST be measured in that unit.
- **FR-017**: The anchor's read-back sentence (spec 008, FR-002b) MUST write the anchor date and base date in their calendars' main formats.

**Storage, terms and layering**

- **FR-018**: Formats and each calendar's main-format choice MUST be stored in the project file as recorded in `/storage/schema.md` (updated by this feature's design PR), added by a numbered migration from schema version 3, and come back exactly as entered. A migrated file's calendars MUST have no formats, so each uses its generated format.
- **FR-019**: Writing and reading dates MUST use exact whole-number arithmetic (principle III) and live below the app and the UI, in the formats component.
- **FR-020**: Labels and messages MUST use the calendar model's terms (ADR 0009), and the Earth-terms check MUST keep passing.

### Key Entities

- **Format**: a named pattern belonging to one calendar, with its place in the calendar's list. Either readable or write-only (worked out from the pattern, with the reason).
- **Field**: a part of a pattern naming a unit or the era, with how its value is written: number, padded number (width), member name, era name.
- **Main format**: the calendar's chosen format for showing dates; the generated format when none is chosen.
- **Generated format**: the format every calendar has without being given one, worked out from its definition.
- **Reading**: one date a piece of text can be read as in a format; reading text gives none, one or several.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The owner can add a format to Aster and see a converter result written in it in under 2 minutes.
- **SC-002**: In a property test of at least 1,000 random dates per format, over the test calendars' generated formats and a set of owner-style patterns (padded, named, separator-free with padding, with units above the era's unit), writing a date and reading it back gives the original date every time.
- **SC-003**: Every kind of pattern refusal (FR-003), reading failure (FR-010), disagreement (FR-012) and ambiguity (FR-011) is covered by a test, and each is reported with its reason; none is resolved silently.
- **SC-004**: A version-3 file opens and migrates with its backup made, and every date the app shows in it is written as before (the generated formats match today's text).
- **SC-005**: Writing and reading a date takes under 10 milliseconds for a calendar with a 1,000-member grouping, so checking as the owner types feels immediate.
- **SC-006**: The Earth-terms, architecture and privacy checks pass.

## Assumptions

- **Kept small (principle VIII)**: locales and name sets, abbreviations, ordinals, roman numerals and words, conditionals, computed values, zone tokens, duration formats, parallel cycles' fields, the full format editor (syntax highlighting, round-trip report over a range) and the universe's display format all come later. The `/formats/templates.md` syntax is the long-term target; this feature builds the subset above in a way that the rest can extend.
- **One era**: calendars still have one era (spec 007), so `{era}` writes its one name; eras' abbreviations come with multiple eras.
- **Formats live in the calendar editor**: as a section of the calendar's form, checked with it when the calendar is kept (spec 007's model), each with its live sample. There is no separate format editor view yet.
- **The sample date**: the editor's sample is a fixed date of the calendar chosen so that each field shows something meaningful (for example the calendar's first date, or its anchor date when it has one); planning picks which.
- **Value-by-value entry stays where it is**: the anchor's date fields stay value by value (spec 008); only the converter's input becomes text.
- **Knowledge update**: the design PR updates `/formats/templates.md` and `/formats/parsing.md` (what this feature builds, write-only formats, the generated format), `/storage/schema.md` (version 4), and `/product/ui-views.md` (formats in the calendar editor and converter), for the owner to re-verify.
- **Scope check (principle VIII)**: if planning shows the build PR would exceed about 30 minutes of review, the spec is split (for example, formats and their editing first, then using them in the converter and the read-back) before the design PR is opened.
