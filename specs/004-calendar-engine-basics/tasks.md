---
description: "Task list for spec 004, calendar engine basics"
---

# Tasks: Calendar Engine Basics

**Input**: Design documents from `specs/004-calendar-engine-basics/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/uni-calendar-api.md](contracts/uni-calendar-api.md), [quickstart.md](quickstart.md)

**Tests**: Included, and written first (constitution VI, FR-018). Each test task must fail before its implementation task.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [ ] T001 In `crates/uni-calendar/Cargo.toml`, add `publish = false`, the dependency `uni-time = { path = "../uni-time" }`, and `[dev-dependencies]` `proptest`, `num-rational` (with the `num-bigint` feature) and `num-bigint`, at exactly the versions in `crates/uni-time/Cargo.toml`. Add no other runtime dependencies (research R9).
- [ ] T002 Replace `crates/uni-calendar/src/lib.rs` with: a crate doc comment (fixed-length calendars, exact conversion between instants and dates, no floating point, constitution III); keep `#![deny(clippy::float_arithmetic)]`; `mod error; mod def; mod date; mod calendar; mod convert;`; and `pub use` of every public type in the contract. Create the five module files with the contract's types and method signatures, each method body `todo!()`, so the crate builds and tests can be written against it (and fail). Every `todo!()` is gone by T014; the finished engine never panics.

---

## Phase 2: Foundational

**Purpose**: the definition types, date types and errors every story uses, and the shared test calendars.

- [ ] T003 [P] Write `crates/uni-calendar/src/error.rs`: `DefinitionError`, `Limit` and `CalendarError` exactly as in the contract, deriving `Clone, Debug, PartialEq, Eq`, implementing `Display` with the messages in data-model.md (for example "sol 56 is outside 1 to 55 in Third", "date is out of the supported range") and `std::error::Error`. Include unit tests of one message per variant.
- [ ] T004 [P] Write `crates/uni-calendar/src/def.rs` (`Clock`, `CalendarDef`, `FundamentalDef`, `GroupingDef`, `MemberUnitDef`, `MemberDef`, `SubdivisionDef`, `EraDef`) and `crates/uni-calendar/src/date.rs` (`Date`, `InstantDate`, `DateStart`) as plain public structs per the contract, with doc comments quoting the data-model rules (for example `MemberDef::count`: "`> 0`: how many of the unit directly below the grouping"; `FundamentalDef::first_number`: "Default 1"; `SubdivisionDef::first_number`: "Default 0").
- [ ] T005 Write `crates/uni-calendar/tests/common/mod.rs` with:
  - `aster(epoch: Instant) -> CalendarDef`, exactly the spec's Aster: groupings age (one member: 10 decades), decade (one member: 10 years), year (member unit "month"; members Prime, Second, Third, … Twelfth with counts `[56, 56, 55, 56, 56, 55, 56, 56, 55, 56, 56, 55]`); fundamental "sol" of `88775244/1000` s; subdivisions hour (20 parts) and minute (100 parts); era "After Landing" numbering "year";
  - `tessel(epoch: Instant) -> CalendarDef` per research R7: grouping "cycle" (member unit "phase", three unnamed members of 13, 13 and 14 turns); fundamental "turn" of exactly 100,000 s with `first_number: Some(0)`; subdivision "watch" (7 parts); era numbering "cycle";
  - an `EPOCH` constant (an arbitrary instant away from zero, such as `Instant::new(1_234_567_890, 123_456_789)`) and a helper `sols(n)` giving `n` Aster sols as a `Span`.

**Checkpoint**: `cargo build -p uni-calendar --all-targets` and `cargo clippy -p uni-calendar --all-targets -- -D warnings` pass.

---

## Phase 3: User Story 1 - Define a calendar (Priority: P1) 🎯 MVP

**Goal**: a definition is checked and either accepted as a `Calendar` or refused with a reason naming the faulty part.

**Independent Test**: `cargo test -p uni-calendar --test aster define` and `cargo test -p uni-calendar calendar::`.

### Tests (write first; they must fail)

- [ ] T006 [US1] In `crates/uni-calendar/tests/aster.rs`, write a `define` module covering US1's acceptance scenarios:
  - Aster is accepted; `units()` is `[age, decade, year, month, sol, hour, minute]`; `era_unit()` is "year"; `units_per("year", "sol")` = 668, `units_per("decade", "sol")` = 6,680, `units_per("sol", "minute")` = 2,000, `units_per("year", "month")` = 12; `units_per("month", "sol")` → `NotFixed`; `units_per("week", "sol")` → `UnknownUnit`;
  - Aster with its sol set to exactly 100,000 s is accepted, with no warning of any kind (FR-002a; scenario 7).
- [ ] T007 [US1] In `crates/uni-calendar/src/calendar.rs`, write unit tests for every refusal in data-model.md's `DefinitionError` table, each asserting the exact variant and the unit (and member) it names:
  - fundamental length 0 and −1 s → `NonPositiveLength`; a grouping with no members → `NoMembers`; a member count of 0 or −3 → `NonPositiveCount` naming the member by name, or by number when unnamed; a subdivision into 0 parts → `TooFewParts`; two members and no member unit → `MembersNeedUnit`;
  - a unit name used twice (including a member unit sharing a grouping's name) → `DuplicateUnit`; two members named "Prime" → `DuplicateMember`;
  - era unit "week" → `EraUnitUnknown`; era unit "hour" → `EraUnitBelowFundamental`; a first number on the year or the decade → `FirstNumberNotAllowed`; empty calendar or era name → `EmptyName`;
  - each `Limit` (research R3): a fundamental length of 1 + 1/10¹⁹ s, which is 10⁹ + 1/10¹⁰ ns (`LengthTooFine`); a 1 ns sol split into 2 (`ShorterThanNanosecond`); a million-year sol split into 2⁶⁰ minutes (`TooManyTicks`); groupings whose product exceeds `i64` (`GroupingTooLong`);
  - accepted edge shapes: a calendar with only a fundamental unit (its era numbers it); a year of one 400-sol month; a 1 ns fundamental unit; a fundamental unit of a million years (31,557,600,000,000 s).

### Implementation

- [ ] T008 [US1] Implement `Calendar::new` and the accessor methods (`name`, `clock`, `era_name`, `epoch`, `units`, `era_unit`, `units_per`) in `crates/uni-calendar/src/calendar.rs`: check every rule in data-model.md in the order the table lists them; flatten the chain (data-model.md "The unit chain"); apply the first-number defaults ("Default 1" for groupings, member units and the fundamental unit; "Default 0" for subdivisions); precompute each grouping's length in fundamental units (`i64`, checked), member prefix sums, the fundamental length in nanoseconds as reduced `p/q` (`Ratio` × 10⁹), and the subdivision part counts; check the limits in research R3. Work until T006 and T007 pass.

**Checkpoint**: Story 1 works on its own.

---

## Phase 4: User Story 2 - Instant to date (Priority: P1)

**Goal**: any instant converts to its fixed number and its full date with an exact remainder.

**Independent Test**: `cargo test -p uni-calendar --test aster to_date` and `cargo test -p uni-calendar convert::`.

### Tests (write first; they must fail)

- [ ] T009 [P] [US2] In `crates/uni-calendar/tests/aster.rs`, write a `to_date` module covering US2's acceptance scenarios, with `E = EPOCH`:
  - `E` → `above [1, 1]`, date `[1, 1, 1, 0, 0]`, remainder 0; `fixed_from_instant(E)` = 0;
  - `E + 668 sols` → year 2, month 1, sol 1; `E + 56 sols` → month 2 (Second), sol 1 (a boundary belongs to the unit that starts there);
  - `E − 1 ns` → `above [0, 0]`, date `[0, 12, 55, 19, 99]`, remainder 44.387621999 s (`44387621999/1000000000`); `fixed_from_instant(E − 1 ns)` = −1;
  - start of year 101 → `above [2, 11]`; start of year −100 → `above [-1, -10]`; start of year −9 → decade 0, start of year −10 → decade −1;
  - an instant 1,234.5 s into Aster's sol 1 → the right hour and minute, and minute start + remainder = the instant exactly;
  - with hours from 1 (scenario US1-6): `E` → hour 1;
  - `Instant::new(i64::MAX, 999_999_999)` and `Instant::new(i64::MIN, 0)` with a 1 ns fundamental unit → `OutOfRange` (fixed number beyond `i64`), never a wrapped value.
- [ ] T010 [P] [US2] In `crates/uni-calendar/src/convert.rs`, write unit tests of the internal steps: floor division for negative offsets; the member-unit lookup in prefix sums at the first and last sol of each month; the subdivision remainder chain for a sol split into 7 (an exact fraction, not whole nanoseconds).

### Implementation

- [ ] T011 [US2] Implement `fixed_from_instant` and `instant_to_date` in `crates/uni-calendar/src/convert.rs` per research R4 and R5: offset in `i128` nanoseconds; `fixed = ⌊offset × q / p⌋` with checked arithmetic; walk the groupings top to bottom (top-level index by floor division, member units by prefix-sum search); number units above the era's unit and the era's unit by count from the epoch + 1, units below by position + first number; split the fundamental remainder through the subdivisions; return the remainder as an exact `Ratio` of seconds. Every overflow is `CalendarError::OutOfRange`; nothing panics. Work until T009 and T010 pass.

**Checkpoint**: Stories 1 and 2 work together.

---

## Phase 5: User Story 3 - Date to instant (Priority: P1)

**Goal**: a date converts to the instant it starts at, or is refused with the unit, value and range.

**Independent Test**: `cargo test -p uni-calendar --test aster to_instant` and `cargo test -p uni-calendar --test tessel`.

### Tests (write first; they must fail)

- [ ] T012 [P] [US3] In `crates/uni-calendar/tests/aster.rs`, write a `to_instant` module covering US3's acceptance scenarios:
  - `[1, 1, 1]` → `E`, `rounded_by: None`; `start_of_fixed(0)` → `E`; `start_of_fixed(-1)` → `E − 1 sol`;
  - `[1, 3, 56]` → `InvalidValue { unit: "sol", value: 56, first: 1, last: 55, within: Some("Third") }`;
  - `[1, 13]` (month 1 to 12), `[1, 1, 0]` (sol 1 to 56, within Prime), `[1, 1, 1, 20]` (hour 0 to 19), `[1, 1, 1, 0, -1]` (minute 0 to 99) → `InvalidValue` naming each unit and range; nothing is wrapped;
  - with hours from 1: `[1, 1, 1, 0]` → `InvalidValue` with range 1 to 20;
  - `[1, 2]` → the start of Second (`E + 56 sols`), and `[1]` → `E` (missing units take their first value);
  - `[]` → `EmptyDate`; six values → `TooManyValues { given: 6, max: 5 }`;
  - `[0, 12, 55]` → `E − 1 sol`; years far outside `i64` sols (`[i64::MAX]`) → `OutOfRange`;
  - every date in the `to_date` module converts back to its instant minus its remainder.
- [ ] T013 [P] [US3] Write `crates/uni-calendar/tests/tessel.rs`: Tessel is accepted; `[1, 1, 0, 1]` → `instant` `T + 14,285.714285715 s` and `rounded_by` `Some(5/7_000_000_000)` (quickstart.md); converting that instant back gives `[1, 1, 0, 1]` with remainder 5/7 ns; `[1, 1, 0, 0]` → `rounded_by: None`; `[1, 2, 0]` → start of turn 13; `[1, 1, 13]` → `InvalidValue` (turn 0 to 12); `[0, 3, 13, 6]` → the last watch before the epoch.

### Implementation

- [ ] T014 [US3] Implement `start_of_fixed` and `date_to_instant` in `crates/uni-calendar/src/convert.rs`: check each value against its unit's range (the member's count for units inside a member unit; `within` names the member, by name or number) before any arithmetic; fill missing smaller units with their first values; sum the exact start as a fraction of nanoseconds from the epoch; round up to the next whole nanosecond with `Rounding::Up`, setting `rounded_by` to the exact difference in seconds when it isn't zero (FR-014); every overflow is `OutOfRange`. Work until T012 and T013 pass.

**Checkpoint**: Stories 1 to 3 work; the engine is usable.

---

## Phase 6: User Story 4 - Exact conversion everywhere (Priority: P2)

**Goal**: property tests prove exactness across random calendars and the whole ±100,000-year range.

**Independent Test**: `cargo test -p uni-calendar --test properties`.

### Tests

- [ ] T015 [US4] Write `crates/uni-calendar/tests/properties.rs` with `proptest` at 10,000 cases per property (`ProptestConfig::with_cases(10_000)`). A strategy builds random valid calendars per research R7 (one to four groupings, up to 20 members with counts 1 to 100, zero to three subdivisions of 1 to 100 parts, fundamental lengths with nanosecond denominators up to 10⁹, a random era unit at or above the fundamental unit, random first numbers from −5 to 5) and random instants within ±100,000 years (±3,155,760,000,000 s) of the epoch. Properties:
  - instant → date → `date_to_instant(date)` + remainder = the instant exactly (US4-1), with remainder in `[0, smallest unit length)`;
  - date (random valid values) → instant → date gives the same date (US4-2), and when `rounded_by` is set the instant is less than 1 ns after the exact start and still converts back to the same date (US4-3);
  - `fixed_from_instant(start_of_fixed(n).instant) = n`;
  - ordering: a later instant never gives an earlier date.
- [ ] T016 [US4] In the same file, write the independent oracle (research R7): a reference `instant_to_date` using `num_rational::BigRational` that steps through units one at a time from the epoch, sharing no code with the engine. A property (10,000 cases) compares it with the engine for random calendars and instants within ±50 top-level units of the epoch.

### Implementation

- [ ] T017 [US4] Fix anything T015 and T016 find in `crates/uni-calendar/src/convert.rs` or `crates/uni-calendar/src/calendar.rs`, adding each failing case as a regression test in the matching unit-test module (constitution VI). If nothing fails, record that in the build PR description.

---

## Phase 7: Performance and polish

- [ ] T018 [P] Write `crates/uni-calendar/tests/timing.rs`: a `#[test] #[ignore]` that runs 100,000 `instant_to_date` and 100,000 `date_to_instant` conversions on Aster, half near the epoch and half about 100,000 years away, timing each direction with `std::time::Instant` and asserting each takes under one second (under 10 µs per conversion). No floats (research R8).
- [ ] T019 Run `cargo test -p uni-calendar --release -- --ignored` and record the timings in the build PR description (SC-004).
- [ ] T020 Add doc comments with a short example to `Calendar::new`, `instant_to_date` and `date_to_instant` in `crates/uni-calendar/src/calendar.rs` and `crates/uni-calendar/src/convert.rs`, and update the crate doc in `crates/uni-calendar/src/lib.rs` to say what this feature covers and what comes later (leap rules, cycles, eras).
- [ ] T021 Run the full local verification and record the results in the build PR description:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `cargo deny check advisories bans sources`
  - `python scripts/check_architecture.py`
  - `python -m unittest discover -s scripts/tests`
  - `python scripts/check_okf.py knowledge`
  - `python scripts/check_privacy.py`
  
  Also confirm the architecture check catches floating point in `uni-calendar`: temporarily add `let x: f64 = 1.0;` to `crates/uni-calendar/src/lib.rs`, see the check fail, and revert (SC-005).
- [ ] T022 Open the build PR as the bot from `build/004-calendar-engine-basics`: label `build`, body with `Design: #<design PR>` and `Closes #37`. Move card #37 to **PR review** with `python scripts/board.py move 37 "PR review"`.

---

## Dependencies & Execution Order

- **Setup → Foundational**: T001 → T002 → T003 and T004 (in parallel) → T005.
- **US1** (T006 to T008) needs the foundational phase. Every other story needs a checked `Calendar`, so US1 comes first.
- **US2** (T009 to T011) needs US1. T009 and T010 can be written in parallel.
- **US3** (T012 to T014) needs US1, and uses US2's conversion for its round-trip checks. T012 and T013 can be written in parallel.
- **US4** (T015 to T017) needs US2 and US3.
- **Performance and polish** come last; T018 can be written alongside US4.

Within each story the order is tests, then implementation, then the checkpoint.

## Parallel examples

- After T002: T003 (errors) and T004 (definition and date types) touch different files.
- In US2: T009 (`tests/aster.rs`) and T010 (`src/convert.rs` tests).
- In US3: T012 (`tests/aster.rs`) and T013 (`tests/tessel.rs`).

## Implementation Strategy

US1 alone (checking definitions) is the MVP slice; US2 and US3 make the engine usable; US4 proves it exact everywhere. Everything ships in one build PR of about 1,000 lines, more than half of them tests (plan.md, principle VIII).

## Notes

- Tick each task in `tasks.md` as it's done; the PR policy refuses a build PR with unticked tasks.
- Build PRs may only change `tasks.md` within `specs/`. A design change discovered during the build needs a new design PR.
