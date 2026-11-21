---
description: "Task list for spec 008, epoch anchoring and the converter"
---

# Tasks: Epoch Anchoring and the Converter

**Input**: Design documents from `specs/008-epoch-anchoring/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/anchors-and-converter.md](contracts/anchors-and-converter.md), [quickstart.md](quickstart.md)

**Tests**: Included, and written first (constitution VI). Each test task must fail before its implementation task.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [x] T001 Add the contract's new surface with `todo!()` bodies so tests can be written against it: `Ratio::to_decimal` (`crates/uni-time/src/ratio.rs`); `Calendar::epoch_for` (`crates/uni-calendar/src/convert.rs`) and `Calendar::smallest_unit_length` (`crates/uni-calendar/src/calendar.rs`); a new `crates/uni-store/src/anchor.rs` with `Anchor`, `AnchorKind`, `AnchorSide`, `AnchorError`, `Placement`, `PlacementProblem` and `OpenUniverse::placements`, `check_anchor`, `followers`; `CalendarRecord.anchor` (set to `None` wherever records are built today, so spec 007's tests still compile); re-exports in `crates/uni-store/src/lib.rs`. Every `todo!()` is gone by T011.

---

## Phase 2: Foundational

**Purpose**: the exact arithmetic and the version-3 table. Every story needs them.

- [x] T002 [P] Tests for `Ratio::to_decimal` in `crates/uni-time/src/ratio.rs`: `37/100` at 3 places is `("0.370", true)`; `1/3` is `("0.333", false)`; `2/3` is `("0.666", false)` (truncated, never rounded up); `0` is `("0.000", true)`; `1/1024` at 3 places is `("0.000", false)`; a whole number `5` is `("5.000", true)`; a negative value truncates towards zero. Then implement it with checked `i128` arithmetic only.
- [x] T003 [P] Tests for `Calendar::smallest_unit_length` in `crates/uni-calendar/src/calendar.rs`: Aster's is 88775.244/2000 s exactly; a calendar with no subdivisions gives the fundamental unit's length. Then implement it.
- [x] T004 [P] Tests for `Calendar::epoch_for` in `crates/uni-calendar/tests/aster.rs` (or a new `tests/anchoring.rs` using `tests/common`): putting Tallow's `[1, 1, 1]` at the start of Aster's `[412, 3, 9]` gives an epoch with which Tallow's date starts at exactly that instant; a date before the epoch (`[0]`, `[-3, 2]`); a calendar of length `355/113` whose date starts between two whole nanoseconds still lands exactly on `at` (research R1); an invalid date gives the engine's `InvalidValue`; an `at` near the end of the timeline gives `OutOfRange`. Add a 1,000-case property test to `crates/uni-calendar/tests/properties.rs` on the existing calendar and date generators: for any valid date `X` and instant `t` in range, the calendar rebuilt with `epoch_for(X, t)` has `date_to_instant(X).instant == t`. Then implement `epoch_for` as research R1 describes.
- [x] T005 Migration tests in `crates/uni-store/src/schema.rs`: a new file is version 3 with an `anchor` table; a version-2 file with three calendars migrates to 3, the first has no anchor row and the second and third are `fixed` anchors to the first with dates `[1]`, `[1]` and `at` (0, 0); a version-2 file with no calendars migrates with no rows; the CHECKs refuse an anchor to itself, `at_secs` without `at_nanos`, `at_nanos` of 10⁹, and an `equation` anchor with no `base_calendar_id`; deleting the base calendar sets `base_calendar_id` to NULL and deleting the anchored calendar removes its row. Then add migration 2 → 3 to `MIGRATIONS` exactly as research R3, set `SCHEMA_VERSION = 3`, and update tests that assert version 2 so they still mean the same thing.
- [x] T006 Tests for reading and writing anchors in `crates/uni-store/src/anchor.rs` (unit tests on an in-memory universe): a fixed anchor with `at` and an equation anchor without round-trip through `add_calendar`, `replace_calendar` and `calendar(id)` field for field; `base_name` round-trips; the date text format is exactly `[412,3,9]`; a stored date of `[]`, `[1,]`, `[ 1]`, `1,2` or `[1.5]`, and a universe with two calendars without an anchor, read as `NotAUniverseFile { reason: "it's damaged" }` (data-model "Tables"). Then implement writing and reading the `anchor` row with the records (research R3), the strict date text format, and the at-most-one-reference check on read.

**Checkpoint**: `cargo test -p uni-time -p uni-calendar -p uni-store` passes.

---

## Phase 3: User Story 1 - Anchor a second calendar to the first (Priority: P1) 🎯 MVP

**Goal**: a later calendar is anchored by equating dates, kept in the file, and placed in time.

**Independent Test**: `cargo test -p uni-store anchor`, `cargo test -p uni-app calendars`, and `npm test -- AnchorSection CalendarEditor`.

- [x] T007 [P] [US1] Tests for `placements()` in `crates/uni-store/src/anchor.rs`: Aster alone is placed at the hidden zero; Tallow with a fixed anchor to Aster's `[412, 3, 9]` is placed so that its `[1, 1, 1]` starts at the same instant as Aster's date; a third calendar anchored to Tallow (fixed and equation) is placed through it; an equation chain of three calendars places in any list order; a calendar whose definition is refused has `Definition` or `Length` as its problem, and an equation anchored to it has `BaseUnusable`. Then implement `placements()` (research R4) with `to_def`, `Calendar::new` and `epoch_for`, rebuilding each definition with its worked-out epoch.
- [x] T008 [P] [US1] Tests for `check_anchor` in `crates/uni-store/src/anchor.rs`: a new calendar in an empty universe needs none (`Ok(None)`); in a universe with calendars it needs one (`Missing`); the reference calendar may stay without one, and may be given one: Aster anchored to a later, fixed Tallow is accepted (FR-001a); Tallow's fixed anchor returns the instant of Aster's `[412, 3, 9]`; an equation anchor returns `None`; `BaseMissing`, `BaseSelf`, `DateEmpty` and `DateGap` on each side; an invalid date on each side gives `BaseDate` or `Date` with the engine's error; FR-005b: replacing Tallow's definition with its anchor unchanged keeps the stored `at` even after Aster changed, and a definition that no longer has Tallow's anchor date gives `Date`; changing any of kind, `base`, `date` or `base_date` works the instant out again from Aster as it is now; an unchanged anchor whose base calendar was deleted is accepted, a changed one is `BaseMissing`; an anchor whose instant would put the epoch beyond the timeline's range is `OutOfRange` (research R5). Then implement it (research R5 steps 5–9; step 7 in T019).
- [x] T009 [P] [US1] File and property tests in `crates/uni-store/src/anchor.rs` (with `tempfile` and `proptest`): Aster, Tallow and a third calendar with anchors of both kinds survive `save_as`, drop and `open` unchanged (Story 1, scenario 3); a version-2 file with two calendars opens with a `.v2.universe` copy, its second calendar anchored as in T005, and every date converts to the same instant as before the migration (scenario 5, SC-004). A 1,000-case property test builds a random universe of 2–5 random valid calendars (spec 007's record generator) where each later calendar gets a random anchor (fixed or equation) with a random earlier one as its base, and in a quarter of cases the first calendar is then anchored, fixing the moment, to a random later calendar that doesn't follow it, with random valid dates; it stores, saves and reopens it and checks every anchor's two dates start at the same instant, and that a random date in one calendar, turned into an instant and back in another calendar and then turned into an instant again, gives the same instant (SC-002).
- [x] T010 [P] [US1] Tests in `crates/uni-app/src/calendars.rs`: `CalendarForm` with `anchor` converts to and from `CalendarRecord` (JSON as data-model.md, trailing `null`s dropped); `new_calendar_form` has no anchor in an empty universe and otherwise a fixed anchor with the reference calendar as base (or the first calendar when there's none) and both dates `[1]` (FR-007); `put_calendar` stores Tallow's anchor and its `at`, and refuses each `AnchorError` with a `FieldError` at the contract's location (SC-003), `InvalidValue` placed at the value's index on each side; `get_calendar` returns `CalendarDetails` with `anchor: null` in Aster's form and Tallow's anchor in Tallow's; `list_calendars` gives a placement problem as `problem`; `anchor_bases(Tallow)` lists Aster and Wick, with Wick `"follows"` while it follows Tallow and a refused calendar `"unplaceable"`, and `anchor_bases(null)` lists every calendar (FR-002a); `read_date({ id: Aster }, [412, 3, 9])` gives orbit 412, span 3 with the name Third, sol 9, bell 0, pulse 0, `read_date` with span 13 gives a `DateError` at index 1, `read_date({ form })` checks against a draft and gives `{ definition }` for a draft the engine refuses (FR-002b). Then implement them in `crates/uni-app/src/calendars.rs`, with `DateValue` and its member names shared with the converter (T021): the anchor's form types and conversions, `put_calendar` calling `check_anchor` after spec 007's four steps and storing its `at`, the total `match` from `AnchorError` and `CalendarError` to `Location` (no `_` arm), `CalendarDetails` (`note` stays `null` until T016), and `list_calendars` from `placements()`.
- [x] T011 [US1] Change `get_calendar` in `crates/uni-app/src/lib.rs` to return `CalendarDetails`, and register `anchor_bases` and `read_date` (contract table).
- [x] T012 [P] [US1] UI tests in `ui/src/AnchorSection.test.tsx` and `ui/src/CalendarEditor.test.tsx` (mocked `invoke`): Aster alone shows "Not anchored.", no anchor inputs and no **Anchor to a base calendar** button (Story 1, scenario 1), and the word "reference" appears nowhere; with Tallow also in the universe, Aster's editor has the button, which opens the anchor inputs starting as for a new calendar (FR-001a), and there's never a way to remove a kept anchor; a new calendar's editor shows **Place in time** with **Base calendar** set to Aster, **Anchor date** and **Base date** holding `1` in their first field, labelled with each calendar's units from the era's unit down (the anchor date's from the draft, the base date's from `converter_calendars`), and "Fix the moment" chosen (FR-007); changing the base calendar resets the base date's fields to its units; the base calendar is a list, with Wick greyed out and unchoosable, labelled "follows this calendar", and a refused calendar labelled "can't be placed" (FR-002a); typing span 13 in the base date calls `read_date` and flags "span 13 is outside 1 to 12" at that field before any keep; with valid dates the read-back reads "Tallow's cycle 1, round 1, watch 1 began at the start of Aster's orbit 412, span 3 (Third), sol 9.", and there's none while a date has a fault; with a `{ definition }` result the anchor date says "Checked once the calendar's definition is valid." (FR-002b); entering Tallow's anchor and keeping sends the expected `anchor` in `put_calendar` (scenario 2); a field holding `"3a"` shows "must be a whole number" without calling the core; a `FieldError` at each anchor location is shown at its field; nothing in the editor renders a timeline value (FR-002, FR-012).
- [x] T013 [US1] Implement `ui/src/AnchorSection.tsx` (research R7), the anchor in `ui/src/calendarDraft.ts` (with a helper giving a draft's units from the era's unit down, tested in `ui/src/calendarDraft.test.ts`) and `ui/src/calendarApi.ts` (types and the new command wrappers), and place the section after the calendar's name in `ui/src/CalendarEditor.tsx`. `converter_calendars` (T020) is needed by the real app here, so register a minimal version of it now if T020 isn't done yet.

**Checkpoint**: Story 1 works end to end (quickstart steps 1–3 and 11–12).

---

## Phase 4: User Story 3 - Anchors stay consistent when calendars change (Priority: P2)

**Goal**: editing, re-anchoring and deleting calendars never leaves one silently out of place. (Done before Story 2 because it finishes the store's anchor behaviour; the converter then shows its results.)

**Independent Test**: `cargo test -p uni-store anchor`, `cargo test -p uni-app calendars`, and `npm test -- AnchorSection CalendarEditor`.

- [x] T014 [P] [US3] Tests in `crates/uni-store/src/anchor.rs`: Tallow fixed to Aster stays at its instant when Aster's spans change (FR-005); Tallow keeping the equation moves so its date still starts at Aster's date (Story 3, scenario 3); an equation whose Aster date no longer exists after Aster's edit has the `BaseDate` problem and every calendar keeping an equation with it has `BaseUnusable` (scenario 2, FR-006); an equation loop stored by raw SQL gives every calendar on it the `Loop` problem naming them in order, and a file holding one still opens (FR-015). Then make `placements()` pass them.
- [x] T015 [P] [US3] Tests for `delete_calendar` and `followers` in `crates/uni-store/src/anchor.rs`: with Tallow keeping an equation with Aster and Wick fixed to Aster, `followers(Aster)` is `["Tallow"]`; deleting Aster turns Tallow's anchor fixed at the instant it had, writes `base_name: "Aster"` on both, and every remaining calendar's dates convert to the same instants as before (FR-006a); deleting the reference calendar leaves no reference, and `new_calendar_form` then anchors to the first calendar in the list; a follower that couldn't be placed gets `at: None` and the `NoMoment` problem; anchoring the reference calendar (FR-001a) leaves no calendar without an anchor, and a file in that state reads back fine. Then implement `followers` and `delete_calendar` per research R6, in one transaction.
- [x] T016 [US3] In `crates/uni-app/src/calendars.rs` and `lib.rs`: tests then implementation of `CalendarDetails.note` (`moved`, with the base calendar's `ConvertedDate` for the stored `at`, when that date differs from `base_date` or isn't at its start; `gone` with `base_name`; `null` otherwise), `CalendarDetails.rounded` (true for an anchor to the `355/113` calendar at a date whose start is rounded, false for Tallow's), and the `calendar_followers` command. `ConvertedDate` is built by T021's function; write that function first if T021 isn't done.
- [x] T017 [P] [US3] UI tests: the anchor's kind choice is sent as `"fixed"` or `"equation"`; a `moved` note reads "Aster has changed since: this moment is now <date> in it." with the date at partway precision, and a `gone` note reads "Its base calendar, Aster, has since been deleted."; `rounded: true` shows "Aster's date starts between two of the timeline's finest steps; this calendar is placed from the next one."; **Delete calendar** first calls `calendar_followers` and, when there are any, the confirmation names them ("Tallow follows Aster and will be fixed where it stands."); a loop `FieldError` is shown at the base calendar (Story 3, scenario 1).
- [x] T018 [US3] Implement the kind choice, the notes and the delete confirmation in `ui/src/AnchorSection.tsx` and `ui/src/CalendarEditor.tsx`.
- [x] T019 [US3] Loop check on confirm: tests in `crates/uni-store/src/anchor.rs` that, while Wick follows Tallow (directly, and through a third calendar), `check_anchor` refuses Tallow with Wick as its base with `Loop`, for both kinds; that Aster (the reference calendar) can't take a base that follows it; and that a base that's merely fixed to the anchored calendar is accepted. Then implement research R5 step 7.

**Checkpoint**: quickstart steps 7–10 work, except for what the converter shows.

---

## Phase 5: User Story 2 - Convert a date between calendars (Priority: P1)

**Goal**: a date entered in one calendar is shown in every other.

**Independent Test**: `cargo test -p uni-app converter` and `npm test -- Converter convertedDate`.

- [x] T020 [P] [US2] Tests in a new `crates/uni-app/src/converter.rs`: `converter_calendars` lists Aster with units `["orbit", "span", "sol", "bell", "pulse"]` and Tallow with `["cycle", "round", "watch"]`, and a calendar that can't be placed with its problem. Then implement it and register it in `crates/uni-app/src/lib.rs` (replacing any minimal version from T013).
- [x] T021 [P] [US2] Tests in `crates/uni-app/src/converter.rs`: `convert(Tallow, [1, 1, 1])` gives Aster with `above` age 5 and cohort 42, values orbit 412, span 3, sol 9, bell 0, pulse 0 and `partway: null` (Story 2, scenario 1); `convert(Aster, [412, 3, 9, 7])` gives Tallow partway with the decimal and exact forms of research R10, `decimalExact` false for a fraction like 1/3 (scenarios 2–3); a date before the epochs gives astronomical numbers (scenario 5); span 13 gives a `DateError` with the engine's message and `index: 1`, and a gap gives one at the gap (scenario 4, FR-011); an unusable calendar is listed in `unusable` with its problem and not in `results` (FR-006); a single calendar gives empty `results`; a source date whose start was rounded gives `rounded: true` (the `355/113` calendar); no field of the JSON is a timeline value (FR-012). Then implement `convert` (research R9, R10), building `ConvertedDate` with `smallest_unit_length` and `to_decimal(3)`, and register it.
- [x] T022 [P] [US2] UI tests in `ui/src/convertedDate.test.ts`: rendering a `ConvertedDate` at each precision ("…, pulse 41, partway through"; "pulse 41.370" and "pulse ≈ 41.333"; "pulse 41 + 37/100"), with units above the era's unit first and no mark when `partway` is null. Then implement `ui/src/convertedDate.ts`.
- [x] T023 [P] [US2] UI tests in `ui/src/Converter.test.tsx`: the list's **Converter** button is disabled until the universe has a calendar that can be placed (FR-008), opens the converter, and **Back to calendars** returns; choosing a calendar shows one field per unit from `converter_calendars`; typing a whole number in each field calls `convert` and shows each result with unit names and member names in brackets (FR-009); a field holding `"7b"` shows "must be a whole number" and doesn't call the core; a `DateError` is shown at its field and no results are shown (FR-011); with one calendar it says "There's no other calendar to convert into."; unusable calendars are listed with their problems; `rounded: true` shows "This date starts between two of the timeline's finest steps; it's shown from the next one."; the precision switch re-renders every result without calling the core, keeps its setting after going back to the list and returning, and starts at "partway" in a freshly mounted `App` (FR-010, FR-010a). Then implement `ui/src/Converter.tsx`, the **Converter** button in `ui/src/CalendarList.tsx`, and the view and the precision state in `ui/src/App.tsx`. Styles in `ui/src/styles.css`.

**Checkpoint**: quickstart steps 4–6 work, and steps 7–10's converter checks.

---

## Phase 6: Polish and verification

- [x] T024 [P] Add an `#[ignore]` timing test `convert_twenty_calendars_timing` in `crates/uni-app/src/converter.rs`: 20 calendars, one with a 1,000-member grouping, anchored in a chain of equations, saved and reopened; time one `convert` from the last calendar. Run it in release and record the result here (SC-005: under 1 s). **Result (2026-11-19, release build): 9.1 ms.**
- [x] T025 [P] Update doc comments: `crates/uni-store/src/lib.rs` (anchors from schema version 3; the terms anchored calendar, base calendar, follows and reference calendar), the epoch comments in `crates/uni-store/src/calendar.rs` that say "until spec 008", and the crate docs of `uni-calendar` for `epoch_for`.
- [x] T026 Run the app by hand through every step of [quickstart.md](quickstart.md) and record the result here, including how long steps 3–4 (anchoring Tallow and converting) took (SC-001: under 3 minutes). **Result (2026-11-21, owner): every step passed. Steps 3–4 weren't timed, so SC-001 isn't measured.**
- [x] T027 Run the full local verification and record the results in the build PR description:
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
- [ ] T028 Open the build PR as the bot from `build/008-epoch-anchoring`: label `build`, body with `Design: #<design PR>` and `Closes #58`. Move card #58 to **PR review** with `python scripts/board.py move 58 "PR review"` only if it is in Building.

---

## Dependencies & Execution Order

- **Setup → Foundational**: T001 → T002, T003 and T004 (in parallel) → T005 → T006.
- **US1** needs the foundational phase: T007, T008 and T009 can be written in parallel (T009 needs T007 and T008 to pass); T010 → T011; T012 → T013.
- **US3** needs US1. T014 and T015 can be written in parallel; T016 after T015; T017 → T018; T019 after T008.
- **US2** needs US1's placements. T020 and T021 can be written in parallel; T022 and T023 after T021's shapes exist (they're mocked in tests).
- **Polish** comes last; T024 and T025 can be written alongside US2.

Within each story the order is tests, then implementation, then the checkpoint.

## Parallel examples

- After T001: T002 (`uni-time`), T003 and T004 (`uni-calendar`) touch different files.
- In US1: T007, T008 (`uni-store`), T010 (`uni-app`) and T012 (UI tests).
- In US2: T020 and T021 (`uni-app`), T022 (UI).

## Implementation Strategy

US1 is the MVP: a second calendar can be anchored and kept. US3 completes how anchors behave when calendars change, and US2 adds the converter, which shows all of it. All ship in one build PR of about 3,300 lines, over a third of them tests (plan.md, principle VIII). If review shows that's too much, the converter (Phase 5 and T024) can become a second build PR, as research R12 notes.

## Notes

- Tick each task in `tasks.md` as it's done; the PR policy refuses a build PR with unticked tasks.
- Build PRs may only change `tasks.md` within `specs/`. A design change discovered during the build needs a new design PR.
