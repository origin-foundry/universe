---
description: "Task list for spec 009, date formats"
---

# Tasks: Date Formats

**Input**: Design documents from `specs/009-date-formats/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/formats.md](contracts/formats.md), [quickstart.md](quickstart.md)

**Tests**: Included, and written first (constitution VI). Each test task must fail before its implementation task.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [ ] T001 Add the contract's new surface with `todo!()` bodies so tests can be written against it: `uni-calendar` and `uni-time` as path dependencies of `crates/uni-format/Cargo.toml` (with `publish = false`) and `proptest` as its dev-dependency; `crates/uni-format/src/pattern.rs` (`Format`, `Part`, `WriteOnly`, `PatternError`, `PatternErrorKind`, `Format::compile`, `generated`, `write_only`, `smallest_shown`), `src/write.rs` (`Format::write`), `src/read.rs` (`Reading`, `ReadError`, `Format::read`), re-exported from `src/lib.rs`; test fixtures for `uni-format` (Aster and Tallow as in spec 008, a calendar with partly named members, and a Gregorian-shaped calendar in a Gregorian-named file, ADR 0009) in a `#[cfg(test)]` module `crates/uni-format/src/fixtures.rs` (the Gregorian-shaped one in `src/fixtures/gregorian.rs`), so every `uni-format` test is a unit test inside the crate; `Calendar::member_names` (`crates/uni-calendar/src/calendar.rs`) and `Calendar::fraction_through` (`crates/uni-calendar/src/convert.rs`); `FormatRecord`, `CalendarRecord.formats` (empty) and `main_format` (`None`) in `crates/uni-store/src/calendar.rs`, set wherever records are built today so spec 007–008 tests still compile; `uni-format` as a dependency of `crates/uni-app/Cargo.toml`. Run `python scripts/check_architecture.py`: it passes, as every new edge is in the allowed list. Every `todo!()` is gone by T013.

---

## Phase 2: Foundational

**Purpose**: the engine accessors and the version-4 table. Every story needs them.

- [ ] T002 [P] Tests for `Calendar::member_names` in `crates/uni-calendar/src/calendar.rs`: Aster's `"span"` gives its twelve names in order; a member unit with some unnamed members gives `None` at those places; `"orbit"`, `"sol"`, `"bell"` and an unknown name give `None`. Then implement it from `Kind::Member { names, .. }`.
- [ ] T003 [P] Tests for `Calendar::fraction_through` in `crates/uni-calendar/tests/aster.rs` (or a new `tests/fractions.rs` using `tests/common`): at the start of orbit 412, span 3, sol 9 it is 0 for every unit; at bell 10 of that sol it is 1/2 of the sol; halfway through a 56-sol span it is 1/2 of the span, and through a span of a different length its own fraction; through an orbit whose length an adjustment rule changed, the fraction uses that orbit's length; a unit above the era's unit (cohort) counts orbits of their own lengths; dates before the epoch; `UnknownUnit` for `"tide"`. Add a 1,000-case property test to `crates/uni-calendar/tests/properties.rs` on the existing generators: for the smallest unit, `fraction_through` equals `instant_to_date(t).remainder / smallest_unit_length()`, and for every unit it is in `[0, 1)`. Then implement it in ticks as research R2 describes, with checked arithmetic.
- [ ] T004 Migration tests in `crates/uni-store/src/schema.rs`: a new file is version 4 with a `format` table and the `format_one_main` index; a version-3 file with calendars migrates to 4 with no format rows; the constraints refuse a second `main = 1` row for one calendar, two formats with the same name in one calendar, `main` of 2, and a repeated `seq`; deleting a calendar deletes its formats. Then add migration 3 → 4 to `MIGRATIONS` exactly as research R7, set `SCHEMA_VERSION = 4`, and update tests that assert version 3 so they still mean the same thing.
- [ ] T005 Tests for formats in records in a new `crates/uni-store/src/format.rs` (unit tests on an in-memory universe): a calendar with three formats and `main_format: Some(1)` round-trips through `add_calendar`, `replace_calendar` and `calendar(id)`, names, templates (exactly as typed, spaces kept) and order unchanged; `main_format: None` writes no `main = 1` row; `normalised()` trims format names and keeps templates; a version-3 file opened from disk gets a `.v3.universe` copy and every calendar reads back with no formats (FR-018, SC-004). Then implement writing and reading `format` rows with the record in `crates/uni-store/src/calendar.rs` and `format.rs`.

**Checkpoint**: `cargo test -p uni-calendar -p uni-store` passes.

---

## Phase 3: User Story 1 - Define a format and see dates in it (Priority: P1) 🎯 MVP

**Goal**: a calendar has named formats and a main format, checked as typed, written in samples, kept in the file.

**Independent Test**: `cargo test -p uni-format pattern write`, `cargo test -p uni-app calendars`, and `npm test -- FormatsSection CalendarEditor`.

- [ ] T006 [P] [US1] Pattern tests in `crates/uni-format/src/pattern.rs`, using Aster, Tallow and a test calendar with partly named members: each `PatternErrorKind` of research R4, with its position (`{orbit` at 0; `a}b` at 1; `{}`; `{tide}` → `UnknownUnit`; `{span:abbr}`; `{span:0}` and `{span:019}` → `BadWidth`; `{sol:name}` and `{orbit:name}` → `NoNames`; `{era:02}` → `EraModifier`; `just text` → `NoField`; a unit named `a:b` → `UnnameableUnit`), and the first fault wins; `{{` and `}}` compile to literal braces; unit names with spaces (`{deep turn:02}`) and spaces around the name compile; `{era}` names the era, unless the calendar has a unit called `era`; `{cohort}` compiles. Write-only (research R5): `{sol} {span:name}` is `Missing { unit: "orbit" }`, `{orbit}.{sol}` is `Gap { shown: "sol", missing: "span" }`, `{orbit}.{span}` and `{span:name} {orbit} {cohort}` are readable; messages as the contract's table. `smallest_shown` for each. Then implement `Format::compile`, `write_only` and `smallest_shown`.
- [ ] T007 [P] [US1] Writing tests in `crates/uni-format/src/write.rs`: Aster's orbit 412, span 3, sol 9 in `{orbit}.{span:02}.{sol:02}` is `412.03.09`; in `{sol} {span:name}, orbit {orbit} {era}` is `9 Third, orbit 412 After Landing`; `{cohort}` writes 42 and `{age}` 5; padding never cuts (`{orbit:02}` of 412 is `412`) and goes after the sign (`{orbit:03}` of −7 is `-007`, of 0 is `000`); a partly named member unit's `:name` writes the number for an unnamed member; the generated format (research R3) of Aster writes `orbit 412, span 3 (Third), sol 9, bell 0, pulse 0`, of Tallow `cycle 1, round 1, watch 1`, and of the partly named calendar no names; the generated format is readable. Then implement `Format::write` and `Format::generated`.
- [ ] T008 [US1] Tests in `crates/uni-app/src/calendars.rs`: `CalendarForm` with `formats` and `mainFormat` converts to and from `CalendarRecord` (JSON as the contract); `put_calendar` stores formats and refuses, at `{ format: { index, field } }` with the contract's messages, an empty name, a name repeated case-insensitively, and each pattern error; a draft whose main format index is out of range is refused; `check_format({ form }, template)` gives the sample (the draft's anchor date when it has a valid one, else its first date, research R8), `writeOnly` with the reason, the error with `at`, or `{ definition }` for a draft the engine refuses; `check_format({ id })` works for a stored calendar. Then implement them, and register `check_format` in `crates/uni-app/src/lib.rs`.
- [ ] T009 [P] [US1] UI tests in a new `ui/src/FormatsSection.test.tsx` and `ui/src/CalendarEditor.test.tsx` (mocked `invoke`): a calendar with no formats shows **Formats** with "Generated" chosen as the main format and an **Add format** button; adding a format gives a row with name and pattern fields, and typing a pattern calls `check_format` (debounced as `read_date` is) and shows the sample, or the error at the pattern, or "Checked once the calendar's definition is valid."; a write-only format shows its reason under the sample; choosing a format as main sends its index as `mainFormat` on keep, and deleting it (or a format before it) adjusts or clears `mainFormat` (data-model "Lifecycle"); a `FieldError` at a format's name or pattern is shown there; keeping sends `formats` exactly as typed. Then implement `ui/src/FormatsSection.tsx`, the formats in `ui/src/calendarDraft.ts` and `ui/src/calendarApi.ts` (types and the `checkFormat` wrapper), and place the section after **Place in time** in `ui/src/CalendarEditor.tsx`. Styles in `ui/src/styles.css`.

**Checkpoint**: quickstart steps 2–4 and 9 work.

---

## Phase 4: User Story 2 - Read text back into a date (Priority: P1)

**Goal**: every readable format reads text into every date it can mean, with reasons for what it can't.

**Independent Test**: `cargo test -p uni-format`.

- [ ] T010 [P] [US2] Reading tests in `crates/uni-format/src/read.rs` (Aster unless named): `412.03.09` and `412.3.9` in `{orbit}.{span:02}.{sol:02}` read as `[412, 3, 9]`; `9 third, orbit 412 after landing` in the long format, with extra spaces and different capitals, reads the same; leading and trailing spaces are ignored; `−7` (U+2212) and `-7` read as −7; `{orbit}{span}` reads `41211` as both `[4121, 1]` and `[412, 11]` and `4123` as only `[412, 3]` (spec US2 scenario 5); `{orbit}{span:02}` reads `41203` as only `[412, 3]` (exact width when touching); a pattern stopping at the span reads `[412, 3]`; `{span} ({span:name})` reads `3 (Third)` and reports `Disagree` for `3 (Fourth)`; `{cohort}` reports `Disagree` when it doesn't match the orbit (scenario 6); a partly named unit reads a name or a number; every `ReadError`: `Empty` for `"  "`, `WriteOnly` for a write-only format, `NoMatch` with the furthest position and the contract's wording for `412-03-09` (at 3, expected `"."`), an unknown member name (expected the member names), trailing text (expected the end of the text) and a number too large for an `i64`, and `Refused` with the engine's error for `412.13.01` (scenario 4); a date before the epoch reads back with its negative values (scenario 7); the same date reached two ways is one reading. Then implement `Format::read` as research R6 describes.
- [ ] T011 [P] [US2] Round-trip property tests in a new `#[cfg(test)]` module `crates/uni-format/src/round_trip.rs` (1,000 cases each, SC-002), on Aster, Tallow and the Gregorian-shaped test calendar (T001's fixtures), with random dates from the calendars' full ranges, including before the epoch: the generated format, and the patterns `{orbit}.{span:02}.{sol:02}`, `{sol} {span:name}, orbit {orbit} {era}`, `{orbit:05}{span:02}{sol:02}`, `{cohort}/{orbit}/{span}` and `{span} ({span:name}) {orbit}` (with each calendar's own unit names in place of Aster's), write then read gives exactly one reading, equal to the date cut to the units shown; `{orbit}{span}{sol}` (unpadded, touching) always has the original among its readings; random text never panics `read`. Make them pass.
- [ ] T012 [US2] An `#[ignore]` timing test `write_read_timing` in a `#[cfg(test)]` module `crates/uni-format/src/timing.rs`: a calendar with a 1,000-member named grouping; time one `write` and one `read` of a date in its last member with a named pattern. Run it in release and record the result here (SC-005: each under 10 ms). **Result:** _(to fill in)_

**Checkpoint**: `cargo test -p uni-format` passes; the crate is complete.

---

## Phase 5: User Story 3 - Formats where the app shows dates (Priority: P2)

**Goal**: converter results and the anchor's read-back sentence are written in each calendar's main format. (Its own phase, so it can move to a second build PR if the build runs long, research R12.)

**Independent Test**: `cargo test -p uni-app converter calendars` and `npm test -- Converter convertedDate AnchorSection`.

- [ ] T013 [US3] Tests in `crates/uni-app/src/converter.rs`: `convert(Tallow, [1, 1, 1])` with Aster's main format generated gives `text` `orbit 412, span 3 (Third), sol 9, bell 0, pulse 0`, `generated: true`, `unit` `"pulse"`; with an owner's main format `{sol} {span:name}, orbit {orbit} {era}` gives `9 Third, orbit 412 After Landing`, `generated: false`, `unit` `"sol"`, and a moment partway through the sol gives `partway` measured in sols (FR-016) in all three forms; a stored main format that no longer compiles falls back to the generated format (research R9); `read_date` gives `text` in the calendar's main format, and for a draft in the draft's main format (FR-017). Then implement them in `crates/uni-app/src/converter.rs` and `calendars.rs` with `Format::write`, `smallest_shown` and `Calendar::fraction_through`.
- [ ] T014 [P] [US3] UI tests in `ui/src/convertedDate.test.ts`, `ui/src/Converter.test.tsx` and `ui/src/AnchorSection.test.tsx`: a `ConvertedDate` with `generated: true` shows its units above the era's unit, then `text`, as before; with `generated: false` it shows only `text`; partway marks read "partway through the sol", "≈ 0.333 of the way through the sol" (and without "≈" when exact) and "1/3 of the way through the sol"; no mark when `partway` is null; the converter still takes its date value by value (US3 scenario 4); the read-back sentence reads "Tallow's cycle 1, round 1, watch 1 began at the start of Aster's 9 Third, orbit 412 After Landing." from the two `text`s. Then implement them in `ui/src/convertedDate.ts`, `ui/src/calendarApi.ts` and `ui/src/AnchorSection.tsx` (the `readBack` function).

**Checkpoint**: quickstart steps 1 and 5–8 work.

---

## Phase 6: Polish and verification

- [ ] T015 [P] Update doc comments: `crates/uni-format/src/lib.rs` crate docs (what spec 009 builds, the pattern syntax, write-only formats, reading every reading), `crates/uni-store/src/lib.rs` (formats from schema version 4), and `uni-calendar`'s crate docs for `member_names` and `fraction_through`.
- [ ] T016 (owner) Run the app by hand through every step of [quickstart.md](quickstart.md) and record the result here, including how long steps 2 and 5 (adding a format and seeing a converter result in it) took (SC-001: under 2 minutes). **Result:** _(to fill in)_
- [ ] T017 Run the full local verification and record the results in the build PR description:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `cargo deny check advisories bans sources`
  - `npm run typecheck`, `npm run lint`, `npm run format:check` and `npm test` (the UI checks CI runs)
  - `python scripts/check_architecture.py`
  - `python -m unittest discover -s scripts/tests`
  - `python scripts/check_okf.py knowledge`
  - `python scripts/check_privacy.py`
  - `python scripts/check_earth_terms.py`
- [ ] T018 Open the build PR as the bot from `build/009-date-formats`: label `build`, body with `Design: #<design PR>` and `Closes #61`. Move card #61 to **PR review** with `python scripts/board.py move 61 "PR review"` only if it is in Building.

---

## Dependencies & Execution Order

- **Setup → Foundational**: T001 → T002 and T003 (in parallel) → T004 → T005.
- **US1** needs T002 (names) and T005 (records): T006 and T007 can be written in parallel; T008 after T006 and T007; T009 after T008's shapes exist (mocked in tests).
- **US2** needs T006 (compiled formats): T010 and T011 can be written in parallel; T012 after T010.
- **US3** needs US1 and T003: T013 → T014.
- **Polish** comes last; T015 can be written alongside US3.

Within each story the order is tests, then implementation, then the checkpoint.

## Parallel examples

- After T001: T002 (`calendar.rs`) and T003 (`convert.rs`, tests) touch different files.
- In US1: T006 (`pattern.rs`) and T007 (`write.rs`).
- In US2: T010 (`read.rs`) and T011 (`round_trip.rs`).

## Implementation Strategy

US1 is the MVP: formats can be defined, checked and kept. US2 makes every format a parser and proves the round trip, which the constitution requires of every format. US3 puts formats where the app shows dates. All ship in one build PR of about 2,800 lines, over a third of them tests (plan.md, principle VIII). If review shows that's too much, US3 (Phase 5) can become a second build PR, as research R12 notes.

## Notes

- Tick each task in `tasks.md` as it's done; the PR policy refuses a build PR with unticked tasks.
- Build PRs may only change `tasks.md` within `specs/`. A design change discovered during the build needs a new design PR.
