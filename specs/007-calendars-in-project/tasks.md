---
description: "Task list for spec 007, calendars in the project file"
---

# Tasks: Calendars in the Project File

**Input**: Design documents from `specs/007-calendars-in-project/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/calendar-commands.md](contracts/calendar-commands.md), [quickstart.md](quickstart.md)

**Tests**: Included, and written first (constitution VI). Each test task must fail before its implementation task.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [X] T001 In `crates/uni-store/Cargo.toml`, add `uni-calendar = { path = "../uni-calendar" }` and `uni-time = { path = "../uni-time" }` to `[dependencies]` (allowed by `/architecture/crate-layout.md`), `publish = false` under `[package]` (path-only workspace dependencies, see `CLAUDE.md`), and `proptest` (same version as `uni-calendar`'s) to `[dev-dependencies]`. Run `python scripts/check_architecture.py`.
- [X] T002 Add the contract's new surface with `todo!()` bodies so tests can be written against it: `Ratio::parse` and `TimeError::NotANumber` (`crates/uni-time/src/ratio.rs`, `error.rs`); a new `crates/uni-store/src/calendar.rs` with `CalendarRecord`, `FundamentalRecord`, `EraRecord`, `RecordError`, `StoredCalendar` and `CalendarRecord::to_def`; `OpenUniverse::calendars`, `calendar`, `add_calendar`, `replace_calendar`, `delete_calendar` in `crates/uni-store/src/universe.rs`; `StoreError::CalendarName`, `CalendarNameTaken { name }`, `NoSuchCalendar { id }` in `crates/uni-store/src/error.rs`; re-exports in `crates/uni-store/src/lib.rs`. Every `todo!()` is gone by T012.

---

## Phase 2: Foundational

**Purpose**: exact lengths, the version-2 tables, and storing records. Every story needs them.

- [X] T003 [P] Tests for `Ratio::parse` in `crates/uni-time/src/ratio.rs`: `"88775"`, `"88775.244"` (= 88775244/1000), `"355/113"`, `"-3"`, `"+0.5"`, `"007"` accepted exactly; `""`, `" 1"`, `"1 "`, `"1e3"`, `"1,000"`, `"1.2.3"`, `"1/"`, `"/2"`, `"1.5/2"`, `"abc"` refused with `NotANumber`; `"1/0"` with `DivisionByZero`; a 40-digit number with `Overflow`. Then implement it with checked `i128` arithmetic only (exact crate, no floats) and the `NotANumber` message "not a whole number, decimal or fraction".
- [X] T004 [P] Migration tests in `crates/uni-store/src/schema.rs`: a new file is version 2 with tables `calendar`, `unit`, `unit_member`, `rule`, `era`; a version-1 file migrates to 2 keeping its universe name; `PRAGMA foreign_keys` is on for new and opened universes. Then add migration 1 → 2 to `MIGRATIONS` with the tables in research R2 exactly (columns, `UNIQUE`, `ON DELETE CASCADE`, the `kind` `CHECK`, and CHECKs that "`length` is set exactly when `kind='fundamental'`; `parts` is set exactly when `kind='subdivision'`; `member_unit_*` is set only when `kind='grouping'`"), set `SCHEMA_VERSION = 2`, and switch foreign keys on wherever `OpenUniverse` creates or opens its connection. Update spec 003's tests that assert version 1, and the version-0 fixture path, so they still mean the same thing.
- [X] T005 Tests for `CalendarRecord::to_def` in `crates/uni-store/src/calendar.rs`: Aster's record (as in quickstart step 4, length `"88775.244"`) converts to spec 004's Aster with the rule and `epoch: Instant::ZERO`; a bad length gives `RecordError::Length` with the text and `NotANumber`; optional fields stay `None`. Then implement `to_def` (research R1, R3, R6).
- [X] T006 Tests for storing records in `crates/uni-store/src/calendar.rs` (unit tests on an in-memory universe): Aster round-trips through `add_calendar` and `calendar(id)` field for field; every optional field empty and set; members with and without names; a rule condition with extra spaces and a line break comes back exactly as typed; three calendars come back in insertion order; `add_calendar` refuses a name that is empty after trimming or longer than 200 characters (`CalendarName`) and a name already used (`CalendarNameTaken`); rows that break the shape (no fundamental unit, two eras) read as `NotAUniverseFile { reason: "it's damaged" }`. Then implement writing a record to rows and reading rows back to a record (data-model "Tables" mapping), `calendars()` ordered by `calendar.seq`, `calendar(id)` (`NoSuchCalendar` when absent) and `add_calendar` (new `seq` = max + 1, marks dirty). Names are trimmed; member names empty after trimming become `None` (research R4).

**Checkpoint**: `cargo test -p uni-time -p uni-store` passes.

---

## Phase 3: User Story 1 - Define a calendar and keep it (Priority: P1) 🎯 MVP

**Goal**: add a calendar from the starting form, see it listed, save, reopen, and find it unchanged.

**Independent Test**: `cargo test -p uni-store`, `cargo test -p uni-app calendars`, and `npm test -- CalendarList CalendarEditor`.

- [ ] T007 [P] [US1] File round-trip tests in `crates/uni-store/src/calendar.rs` (with `tempfile`): Aster plus two more calendars survive `save_as`, drop and `open` unchanged and in order (Story 1, scenarios 2–3); a version-1 file opens with no calendars and its `.v1.universe` copy (scenario 4, SC-005). A property test (`proptest`, 1,000 cases) generating random valid records (1–4 groupings with 1–20 members, optional names and first numbers, 0–3 subdivisions, an era on a random grouping or the fundamental unit, 0–2 rules of the form `<era unit> % k == r` on the era's unit when it's a grouping, lengths as random whole numbers, decimals or fractions accepted by `Calendar::new`) checks store → save → reopen → identical record (SC-002).
- [ ] T008 [P] [US1] Tests in a new `crates/uni-app/src/calendars.rs`: `CalendarForm` ↔ `CalendarRecord` both ways for Aster (JSON in camelCase as in data-model.md); `new_calendar_form` gives "Calendar", fundamental unit "unit" with empty length, era "Era" numbering "unit", no groupings, subdivisions or rules, and "Calendar 2" when "Calendar" is taken (research R7, FR-002a); `put_calendar(None, aster)` adds it, returns the state with `dirty: true` and the new id; `list_calendars` gives `{ id, name: "Aster", problem: null }`; a required number left `null` is refused as missing at its field (FR-011, edge case "Empty fields"). Then implement the module: the form types, the conversions, the missing-value check (research R8 step 1), the starting form, and `Session` methods for list, get, new form and put (add only), following spec 003's `Session` pattern in `crates/uni-app/src/lib.rs`.
- [ ] T009 [US1] Register `list_calendars`, `get_calendar`, `new_calendar_form` and `put_calendar` as Tauri commands in `crates/uni-app/src/lib.rs` (contract table), each returning "No universe is open." without one, and `put_calendar` sending the window-title update the way spec 003's commands do (`reflect`).
- [ ] T010 [P] [US1] UI tests in `ui/src/CalendarList.test.tsx` and `ui/src/CalendarEditor.test.tsx` (mocked `invoke`, as `App.test.tsx` does): with a universe open the main area lists calendars and offers **Add calendar**; **Add calendar** opens the editor on the starting form; the length field is labelled "Length, in SI seconds (physical)" (FR-008); the editor has no epoch field and renders no absolute-timeline value (FR-009); an empty first-number field shows the engine's default (1, or 0 for a subdivision) as greyed hint text and is sent as `null` (FR-007); entering Aster section by section (groupings with members and rules, the fundamental unit, subdivisions, era) and pressing **Keep changes** sends the expected `CalendarForm` and returns to the list showing Aster; lists support add, remove, move up and move down (FR-006); a whole-number field holding `"12a"` shows "must be a whole number" without calling the core; the length is sent as the text typed, never converted to a number.
- [ ] T011 [US1] Implement `ui/src/calendarApi.ts` (types and command wrappers from the contract, telling a `FieldError` from a string error by its `at`), `ui/src/CalendarList.tsx`, `ui/src/CalendarEditor.tsx` (sections per research R10; empty optional fields show the engine's default as hint text (FR-007); labels use the model's terms: grouping, member, member unit, fundamental unit, subdivision, era, era's unit, adjustment rule, condition, change), and show the list or the editor in `ui/src/App.tsx` when a universe is open. Styles in `ui/src/styles.css`.
- [ ] T012 [US1] FR-015: a test in `crates/uni-store/src/calendar.rs` that a stored calendar with a bad length, and one the engine refuses (written with raw SQL), both read back as records; a test in `crates/uni-app/src/calendars.rs` that `list_calendars` gives each its `problem` message and `get_calendar` still returns its form. Then implement the `problem` check in `list_calendars` (`to_def` then `Calendar::new`, research R9), and show a problem in `CalendarList.tsx` beside the calendar's name, with a UI test.

**Checkpoint**: Story 1 works end to end (quickstart steps 1, 2, 4, 6, 9).

---

## Phase 4: User Story 2 - Edit and delete calendars (Priority: P1)

**Goal**: change, discard or delete a stored calendar, and never lose an edit without being asked.

**Independent Test**: `cargo test -p uni-store -p uni-app calendars` and `npm test -- CalendarEditor fileFlows`.

- [ ] T013 [P] [US2] Store tests in `crates/uni-store/src/calendar.rs`: `replace_calendar` keeps the id and position and replaces every unit, member, rule and the era (no orphan rows left: count rows per table); renaming to its own name is allowed, to another calendar's is `CalendarNameTaken`; `delete_calendar` removes the calendar and, by cascade, all its rows; both mark dirty; deleting then closing without saving leaves the file with the calendar (Story 2, scenario 5). Then implement `replace_calendar` (one transaction) and `delete_calendar`.
- [ ] T014 [US2] In `crates/uni-app/src/calendars.rs` and `lib.rs`: tests then implementation of `put_calendar(Some(id), form)` (replace) and the `delete_calendar` command (returns the state, dirty).
- [ ] T015 [P] [US2] UI tests: choosing a listed calendar opens its stored form (`get_calendar`); changing a member's count and keeping sends `put_calendar` with its id (Story 2, scenario 1); **Discard changes** returns to the list without calling the core (scenario 2); **Delete calendar** asks first and calls `delete_calendar` only on yes (scenario 4); with an edit pending, File → Close, New, Open, Exit and the window's close button first ask "Keep or discard your changes to <calendar>?". Keep runs `put_calendar` and stops the flow if it's refused; then the existing unsaved-universe question follows (scenario 3, FR-012). With no changes there's no question.
- [ ] T016 [US2] Implement editing a stored calendar, **Discard changes** and **Delete calendar** in `ui/src/CalendarEditor.tsx`, and the pending-edit guard in front of the flows in `ui/src/fileFlows.ts` and `ui/src/App.tsx` (menu actions, keyboard shortcuts and `close-requested` all pass through it).

**Checkpoint**: quickstart steps 7 and 8 work.

---

## Phase 5: User Story 3 - Invalid calendars are refused with the reason (Priority: P1)

**Goal**: every refusal shows the engine's message at the field it concerns, and the form keeps what was typed.

**Independent Test**: `cargo test -p uni-app calendars::locations` and `npm test -- CalendarEditor`.

- [ ] T017 [P] [US3] One test per `DefinitionError` variant in `crates/uni-app/src/calendars.rs` (module `locations`): build a form that triggers it, call `put_calendar`, and check the `FieldError`'s message equals the engine's `Display` text and its `at` matches the contract's placement table. Cover `DuplicateUnit` on a member unit name, `FirstNumberNotAllowed` on a member unit, `BeyondLimits` on the fundamental unit and on a grouping, and the fallback arm (a name matching no unit gives `calendarName`). Also cover the calendar-name faults (empty, too long, taken) at `calendarName` and the length parser's refusal at `fundamental.length` (research R8, steps 2–3). Nothing is stored in any case (SC-003).
- [ ] T018 [US3] Implement the checking order (research R8) and the total `match` from `DefinitionError` to `Location` in `crates/uni-app/src/calendars.rs`. No `_` arm: a new engine variant must fail to compile.
- [ ] T019 [P] [US3] UI tests: a `FieldError` at each location kind is shown next to that input (or at the top of the section when `field` is absent), the input is marked invalid and focused, the form keeps every value, and fixing it and keeping again succeeds (Story 3, scenarios 1–4); the rule-condition message keeps its position text ("at 1:8").
- [ ] T020 [US3] Implement showing field errors in `ui/src/CalendarEditor.tsx`.

**Checkpoint**: quickstart steps 3 and 5 work.

---

## Phase 6: Polish and verification

- [ ] T021 [P] Add an `#[ignore]` timing test `list_twenty_calendars_timing` in `crates/uni-store/src/calendar.rs`: 20 calendars, one with a 1,000-member grouping, saved, reopened and listed with every calendar checked by the engine (as `list_calendars` does), printing the time. Run it in release and record the result here (SC-004: under 1 s).
- [ ] T022 [P] Update doc comments and the crate docs in `crates/uni-store/src/lib.rs` (calendars are stored from schema version 2) and `crates/uni-time/src/ratio.rs` (`Ratio::parse` with examples), and spec 003's contract note in `crates/uni-store` if it says version 1.
- [ ] T023 Run the app by hand through every step of [quickstart.md](quickstart.md) and record the result here, including how long step 4 (defining Aster) took (SC-001: under 5 minutes) and that native menu shortcuts and the window's close button pass through the pending-edit guard (Vitest can't check this, see `CLAUDE.md`, Tauri).
- [ ] T024 Run the full local verification and record the results in the build PR description:
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
- [ ] T025 Open the build PR as the bot from `build/007-calendars-in-project`: label `build`, body with `Design: #<design PR>` and `Closes #52`. Move card #52 to **PR review** with `python scripts/board.py move 52 "PR review"` only if it is in Building.

---

## Dependencies & Execution Order

- **Setup → Foundational**: T001 → T002 → T003 and T004 (in parallel) → T005 → T006.
- **US1** needs the foundational phase: T007 and T008 can be written in parallel; T008 → T009; T010 → T011 (needs T009's commands for the real app, not for the tests); T012 last.
- **US2** needs US1 (the editor and the commands). T013 and T015 can be written in parallel.
- **US3** needs US1's `put_calendar`; it doesn't need US2. T017 and T019 can be written in parallel.
- **Polish** comes last; T021 and T022 can be written alongside US3.

Within each story the order is tests, then implementation, then the checkpoint.

## Parallel examples

- After T002: T003 (`uni-time`) and T004 (`uni-store` schema) touch different crates.
- In US1: T007 (`uni-store` tests), T008 (`uni-app`) and T010 (UI tests).
- In US3: T017 (`uni-app` locations) and T019 (UI tests).

## Implementation Strategy

US1 is the MVP: a calendar can be made, kept and reopened. US2 makes calendars correctable, and US3 makes every refusal clear. All three are P1 and ship in one build PR of about 2,600 lines, over a third of them tests (plan.md, principle VIII). If review shows that's too much, the storage half (Phases 1–2 and T007) can go first, as research R12 notes.

## Notes

- Tick each task in `tasks.md` as it's done; the PR policy refuses a build PR with unticked tasks.
- Build PRs may only change `tasks.md` within `specs/`. A design change discovered during the build needs a new design PR.
