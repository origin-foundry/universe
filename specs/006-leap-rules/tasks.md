---
description: "Task list for spec 006, leap rules"
---

# Tasks: Leap Rules

**Input**: Design documents from `specs/006-leap-rules/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/uni-calendar-api.md](contracts/uni-calendar-api.md), [quickstart.md](quickstart.md)

**Tests**: Included, and written first (constitution VI, FR-014). Each test task must fail before its implementation task.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [X] T001 In `crates/uni-calendar/Cargo.toml`, add `uni-expr = { path = "../uni-expr" }` to `[dependencies]` (allowed by `/architecture/crate-layout.md`; research R2). No other new dependencies.
- [X] T002 Add the contract's new surface with `todo!()` bodies so tests can be written against it: `LeapRuleDef { condition: String, member: String, change: i64 }` and `GroupingDef.rules: Vec<LeapRuleDef>` in `crates/uni-calendar/src/def.rs`; `EraInstance { members: Vec<i64>, length: i64, holding: Vec<usize> }` in `crates/uni-calendar/src/date.rs`; `EraNumber { unit: String, number: i64 }` in `crates/uni-calendar/src/error.rs`; `pub const RULE_PERIOD_LIMIT: i64 = 100_000;`, `Calendar::rule_period` and `Calendar::era_instance` in `crates/uni-calendar/src/calendar.rs`; a new `crates/uni-calendar/src/rules.rs` module; re-exports in `crates/uni-calendar/src/lib.rs`. Add `rules: vec![]` to every existing `GroupingDef` built in `src/` doc tests and unit tests, `tests/common/mod.rs`, `tests/aster.rs`, `tests/tessel.rs`, `tests/properties.rs` and `tests/timing.rs`, so spec 004's tests still pass. Every `todo!()` is gone by T015.

---

## Phase 2: Foundational

**Purpose**: the errors every story reports, and the test calendars.

- [X] T003 [P] In `crates/uni-calendar/src/error.rs`, add the data model's new `DefinitionError` variants (`RulesNotOnEraUnit { unit }`, `RuleCondition { unit, rule, error: uni_expr::CheckError }`, `RuleUnknownMember { unit, rule, member }`, `RuleZeroChange { unit, rule }`, `RulePeriod { unit, rule, error: uni_expr::PeriodError }`, `RulesPeriodOverLimit { unit, limit }`, `RuleFails { unit, rule, year, error: uni_expr::EvalError }`, `MemberTooShort { unit, member, year, length }`), `Limit::CycleTooLong`, `CalendarError::InvalidValue`'s new `year: Option<EraNumber>` field ("set whenever the calendar has rules and the unit is below the era's unit; `None` otherwise") and `CalendarError::NotAGrouping { unit }`, with the messages in data-model.md (for example "rule 1 of year fails in year 7: division by zero at 1:5", "sol 56 is outside 1 to 55 in Twelfth in year 1", "turn 13 is outside 0 to 12 in 3 in cycle 3"). Update every existing `InvalidValue` construction and match to the new field (`year: None`). Add one message test per new variant to the existing message tests.
- [X] T004 [P] In `crates/uni-calendar/tests/common/mod.rs`, add `aster_with_rule(epoch)` (Aster plus the spec's rule: condition `year % 5 == 0 && (year % 100 != 0 || year % 500 == 0)`, member `Twelfth`, change `1`), `tessel_with_rule(epoch)` (Tessel plus member `3` shortened by 1 when `cycle % 3 == 0`, research R8) and `gregorian_shaped()` (86,400 s day named "day", a "year" of 12 months with member unit "month", counts `[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]`, February lengthened by 1 by the Gregorian rule, era numbering years with its epoch at `Instant::ZERO`).

**Checkpoint**: `cargo build -p uni-calendar --all-targets` and `cargo test -p uni-calendar` pass (spec 004's tests unchanged in meaning).

---

## Phase 3: User Story 1 - Define a calendar with leap rules (Priority: P1) 🎯 MVP

**Goal**: rules are checked with the rest of the definition, the rule period and cycle table are built, and every rule that could fail later is refused with the rule, the reason and a year.

**Independent Test**: `cargo test -p uni-calendar --test leap_rules define_` and `cargo test -p uni-calendar rules::`.

### Tests (write first; they must fail)

- [X] T005 [P] [US1] Write the definition tests in `crates/uni-calendar/tests/leap_rules.rs` (names starting `define_`), one per Story 1 scenario, each asserting the exact error variant and fields:
  - Aster with its rule is accepted; `rule_period()` is 500; `units_per("year", "sol")` and `units_per("decade", "sol")` are `NotFixed`; `units_per("year", "month")` is 12 and `units_per("decade", "year")` is 10; Aster without rules has `rule_period()` 1 and `units_per("year", "sol")` 668;
  - condition `year % 4` → `RuleCondition { rule: 1, error: TypeMismatch { context: "result", .. } }`; `month == 2` → `RuleCondition` with `UnknownVariable { name: "month", at: 1:1 }`;
  - member `Thirteenth` → `RuleUnknownMember { rule: 1, member: "Thirteenth" }`; change 0 → `RuleZeroChange`; a second rule's fault reports `rule: 2`;
  - shorten `Third` by 55 when `year % 2 == 0` → `MemberTooShort { unit: "year", member: "Third", year: 2, length: 0 }`; two rules that each shorten `Third` by 30 when `year % 2 == 0` → `MemberTooShort { length: -5, .. }` (changes add up);
  - `100 / (year % 7) == 3` → `RuleFails { rule: 1, year: 7, error: DivisionByZero { at: 1:5 } }`; `year * 4611686018427387904 % 3 == 0` → `RuleFails { year: 2, error: OutOfRange { .. } }`;
  - a rule on the decade → `RulesNotOnEraUnit { unit: "decade" }`; a rule on the year of a calendar whose era numbers sols → `RulesNotOnEraUnit { unit: "year" }`;
  - `year > 100` → `RulePeriod { rule: 1, error: NotRepeating { at: 1:6 } }`; `year % 100003 == 0` → `RulePeriod` with `OverLimit { limit: 100000 }`; three rules `year % 97 == 0`, `year % 89 == 0`, `year % 83 == 0` (each fine, lcm 716,539) → `RulesPeriodOverLimit { limit: 100000 }`;
  - an era unit named `if` (a keyword) with a rule → `RuleCondition` with `InvalidVariable`; with no rules it is still accepted (spec 004 behaviour);
  - Tessel with its rule is accepted, with `rule_period()` 3, and its condition reads `cycle`;
  - a rule whose member is given by number (`"3"`) on Tessel's unnamed members works; `"4"` is `RuleUnknownMember`.
- [X] T006 [P] [US1] Write unit tests in `crates/uni-calendar/src/rules.rs` for the cycle table: for Aster with its rule, `P = 500`, two variants, residue 4 (year 5) and residue 499 (year 500) leap, residue 99 (year 100) common, the cycle length 334,096 and `starts[5]` 3,341; two rules on one member add up in one variant; a calendar without rules gets `P = 1` and one variant equal to the listed members.

### Implementation

- [X] T007 [US1] Implement rule checking and the cycle table in `crates/uni-calendar/src/rules.rs`, called from `Calendar::new` after spec 004's checks, in research R5's order: rules only on the era's unit, which must be a grouping; each rule's condition checked with `uni_expr::Expr::check(condition, &[VarDecl { name: <era unit name>, ty: Type::Int }], Type::Bool)`, its member resolved by name or number from 1, its change "Not 0", its period `expr.period(<era unit name>, RULE_PERIOD_LIMIT)`; the lcm of the periods (checked, refused over the limit); then for years 1 … P, each condition evaluated (`RuleFails` with the first failing year), member counts after the changes "at least 1" (`MemberTooShort` with the first year), variants keyed by the set of holding rules, member starts in fundamental units and `starts[0 … P]`, all checked arithmetic (`BeyondLimits { limit: CycleTooLong }` on overflow). Store the cycle in `Calendar`; implement `rule_period`. Work until T005 and T006 pass.
- [X] T008 [US1] Update `Calendar::units_per` in `crates/uni-calendar/src/calendar.rs` per research R6: counts that depend on the variant (anything measured in units at or below the era's unit's members across an era unit with rules, such as sols in a year or a decade) are `NotFixed`; counts the rules don't change (months in a year, years in a decade, sols in an hour's parent) are reported as before. Work until T005's `units_per` checks pass and spec 004's still do.

**Checkpoint**: Story 1 works on its own.

---

## Phase 4: User Story 2 - Convert dates in a calendar with leap rules (Priority: P1)

**Goal**: both conversion directions and `era_instance` follow the rules, before and after the epoch.

**Independent Test**: `cargo test -p uni-calendar --test leap_rules convert_`.

### Tests (write first; they must fail)

- [X] T009 [US2] Write the conversion tests in `crates/uni-calendar/tests/leap_rules.rs` (names starting `convert_`), one per Story 2 scenario, on `aster_with_rule`:
  - `[5, 12, 56]` → instant → back gives the same date; its fixed number is 3,340;
  - `[1, 12, 56]` → `InvalidValue { unit: "sol", value: 56, first: 1, last: 55, within: Some("Twelfth"), year: Some(EraNumber { unit: "year", number: 1 }) }`;
  - `[6]` starts at fixed number 3,341; `[501]` at 334,096; `[0]` at −669 (year 0 is a leap year) and `[-4]` at −3,341 (years −4 to −1 common, year 0 leap), worked by hand in the test's comments;
  - one nanosecond before the epoch → `above` `[0, 0]` (age 0, decade 0), date `[0, 12, 56, 19, 99]`, remainder one minute less 1 ns;
  - `[100, 12, 56]` refused with `year: Some(EraNumber { unit: "year", number: 100 })`; `[500, 12, 56]` accepted;
  - `era_instance(5)` → Twelfth 56 (the 12th entry), `length` 669, `holding` `[1]`; `era_instance(1)` → 55, 668, `[]`; `era_instance(0)` and `era_instance(-5)` leap; `era_instance` on a calendar whose era numbers sols → `NotAGrouping`;
  - decades: the instant at the start of year 11 is in decade 2, and the last nanosecond of year 10 in decade 1; year 101 is in age 2;
  - on `tessel_with_rule`: cycles 0, 3 and −3 have phase 3 of 13 turns, cycle 1 of 14; `[3, 3, 13]` (cycle 3, phase 3, turn 13; turns count from 0) is refused with `year: Some(EraNumber { unit: "cycle", number: 3 })` while `[1, 3, 13]` is accepted, and fixed numbers around cycles −3 to 4 match sums worked by hand in the test;
  - two rules that both hold (Aster's rule plus `year % 25 == 0` lengthening Twelfth by 2): `era_instance(25)` has Twelfth 58 and `holding` `[1, 2]`, and `[25, 12, 58]` converts both ways (Story 1 scenario 8, FR-004);
  - a year whose start lies outside the range (`[i64::MAX / 2]` and `[i64::MIN / 2]`) → `OutOfRange`, never a wrapped date (FR-012);
  - spec 004's `aster.rs` and `tessel.rs` still pass, unchanged in meaning (calendars without rules report `year: None`).

### Implementation

- [X] T010 [US2] In `crates/uni-calendar/src/convert.rs`, route both directions through the cycle (research R4, R6): instant to date finds the era unit's count by floored division by the cycle length and a binary search in `starts`, numbers units above the era's unit by that count divided by their length in era-unit instances, and uses the year's variant for the member unit and the units inside it; date to instant checks each value against the year's variant (setting `InvalidValue.year` when the calendar has rules) and adds `cycle × cycle length + starts[r] + member start`. All arithmetic checked (`OutOfRange`). Keep calendars whose era's unit isn't a grouping on spec 004's path. Implement `Calendar::era_instance` in `crates/uni-calendar/src/calendar.rs`. Work until T009 passes and spec 004's tests still do.

**Checkpoint**: Stories 1 and 2 work together.

---

## Phase 5: User Story 3 - Exact conversion everywhere, with rules (Priority: P2)

**Goal**: property tests and the Gregorian comparison prove conversion exact across the range.

**Independent Test**: `cargo test -p uni-calendar --test properties` and `cargo test -p uni-calendar --test gregorian`.

### Tests

- [X] T011 [P] [US3] Write `crates/uni-calendar/tests/data/gregorian_days.py` (standard library only): for each year 1 to 4000, print one line `year jan1_ordinal m1 … m12` using `datetime.date(year, 1, 1).toordinal()` and `calendar.monthrange(year, m)[1]`, after a header comment naming the script, the Python version (`sys.version`) and the source of the numbers. Run it to write `crates/uni-calendar/tests/data/gregorian_days.txt` with `\n` line endings, and commit both (constitution VI: generator and version committed; no published table copied).
- [X] T012 [P] [US3] Write `crates/uni-calendar/tests/gregorian.rs`: load the fixture (`include_str!`), and for every day of years 1 to 4000 of `gregorian_shaped()`, check that `date_to_instant([year, month, day])` gives the fixed number `ordinal − 1` (epoch at 1 January of year 1, ordinal 1), that `instant_to_date` of that instant gives the date back, that the month lengths from `era_instance` match the fixture's, and that day `length + 1` of each month is refused (SC-005).
- [X] T013 [US3] Extend `crates/uni-calendar/tests/properties.rs` (10,000 cases per property, research R8): random calendars from the existing generator whose era's unit is a grouping get zero to three random rules, each condition `(a * <unit> + b) % m == c` (with `m` from 1 to 50, `a` from −3 to 3, `0 ≤ c < m`) optionally joined to a second by `&&` or `||`, a random member, and a change from −2 to 3 chosen so no member can drop below 1. Properties: instant → date → start plus remainder gives the instant back; date → instant → date gives the date back; and dates agree with a **year-by-year reference** in the test file that walks years from the epoch evaluating every rule directly with `uni_expr` (no cycle table), within ±(2 × rule period + 50) years of the epoch. Rules the engine refuses (over the period limit) are skipped, not failed.
- [X] T014 [US3] Fix anything T012 or T013 finds in `crates/uni-calendar/src/`, adding each failing case as a regression test in the matching unit-test module (constitution VI). If nothing fails, record that in the build PR description.

---

## Phase 6: Performance and polish

- [X] T015 [P] Extend `crates/uni-calendar/tests/timing.rs` with `#[ignore]` checks on `aster_with_rule` and `gregorian_shaped()`: 100,000 conversions each way, half near the epoch and half about 100,000 years away (within the calendar's range), each direction under one second (under 10 µs each, SC-004). No floats.
- [X] T016 Run `cargo test -p uni-calendar --release -- --ignored` and record the timings in the build PR description (SC-004).
- [X] T017 Add doc comments with a short example (the contract's) to `LeapRuleDef`, `Calendar::rule_period` and `Calendar::era_instance`, and update the crate doc in `crates/uni-calendar/src/lib.rs`: leap rules on the era's unit now exist; what comes later (rules on other units, inserting members, intercalary members, length and start expressions, non-repeating rules).
- [X] T018 Run the full local verification and record the results in the build PR description:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `cargo deny check advisories bans sources`
  - `python scripts/check_architecture.py`
  - `python -m unittest discover -s scripts/tests`
  - `python scripts/check_okf.py knowledge`
  - `python scripts/check_privacy.py`
  - `python crates/uni-calendar/tests/data/gregorian_days.py` reproduces the committed fixture (compare with `git diff --exit-code` after writing it)

  Also confirm the architecture check catches floating point in `uni-calendar`: temporarily add `let x: f64 = 1.0;` to `crates/uni-calendar/src/lib.rs`, see the check fail, and revert (SC-006).
- [X] T019 Open the build PR as the bot from `build/006-leap-rules`: label `build`, body with `Design: #<design PR>` and `Closes #43`. Move card #43 to **PR review** with `python scripts/board.py move 43 "PR review"` only if it is in Building.

---

## Dependencies & Execution Order

- **Setup → Foundational**: T001 → T002 → T003 and T004 (in parallel).
- **US1** (T005 to T008) needs the foundational phase. T005 and T006 can be written in parallel; T007 → T008.
- **US2** (T009, T010) needs US1 (the cycle table).
- **US3** (T011 to T014) needs US2. T011 and T012 can be written in parallel with T013.
- **Performance and polish** come last; T015 can be written alongside US3.

Within each story the order is tests, then implementation, then the checkpoint.

## Parallel examples

- After T002: T003 (errors) and T004 (test calendars) touch different files.
- In US1: T005 (`tests/leap_rules.rs`) and T006 (`src/rules.rs` tests).
- In US3: T011 (fixture generator), T012 (`tests/gregorian.rs`) and T013 (`tests/properties.rs`).

## Implementation Strategy

US1 alone (defining and checking rules) is the MVP slice; US2 makes calendars with leap years usable; US3 proves them exact everywhere and against an independent implementation. Everything ships in one build PR of about 1,500 reviewable lines, more than half of them tests, plus the generated fixture (plan.md, principle VIII).

## Notes

- Tick each task in `tasks.md` as it's done; the PR policy refuses a build PR with unticked tasks.
- Build PRs may only change `tasks.md` within `specs/`. A design change discovered during the build needs a new design PR.
