---
description: "Task list for spec 002, exact time core"
---

# Tasks: Exact Time Core

**Input**: Design documents from `specs/002-exact-time-core/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/uni-time-api.md](contracts/uni-time-api.md), [quickstart.md](quickstart.md)

**Tests**: Included, and written first (constitution VI). Each test task must fail before its implementation task.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [x] T001 In `crates/uni-time/Cargo.toml`, add `[dev-dependencies]`: `proptest`, `num-rational` (with the `num-bigint` feature) and `num-bigint`, at current versions. Add no runtime dependencies.
- [x] T002 Replace `crates/uni-time/src/lib.rs` with: a crate doc comment (exact moments, spans and fractions; no floating point, constitution III); `#![deny(clippy::float_arithmetic)]`; `mod error; mod rounding; mod ratio; mod span; mod instant;`; and `pub use` of `Instant`, `Span`, `Ratio`, `Rounding`, `TimeError`. Create the five module files as empty placeholders, so the crate still builds.

---

## Phase 2: Foundational

**Purpose**: the error and rounding types every story uses.

- [x] T003 [P] Write `crates/uni-time/src/error.rs`: `TimeError { OutOfRange, DivisionByZero, Overflow }` deriving `Debug, Clone, Copy, PartialEq, Eq`, implementing `Display` with the messages in data-model.md ("time is out of range", "division by zero", "fraction too large to represent exactly"), and `std::error::Error`. Include a unit test of the three messages.
- [x] T004 [P] Write `crates/uni-time/src/rounding.rs`: `Rounding { Down, Up, NearestEven }`, deriving `Debug, Clone, Copy, PartialEq, Eq`, with a doc comment for each direction (Down is towards −∞, Up is towards +∞, NearestEven sends ties to even).

**Checkpoint**: `cargo build -p uni-time` and `cargo clippy -p uni-time -- -D warnings` pass.

---

## Phase 3: User Story 2 - Exact fractions (Priority: P1)

Built before US1 because spans and instants convert to and from fractions (US3), and fractions have no dependencies.

**Independent Test**: `cargo test -p uni-time ratio`.

### Tests (write first; they must fail)

- [x] T005 [US2] In `crates/uni-time/src/ratio.rs`, write unit tests for the contract's `Ratio` API:
  - `new` reduces (2/4 → 1/2; −2/−4 → 1/2; 2/−4 → −1/2) and gives zero as 0/1;
  - `new(_, 0)` and `checked_div` by zero → `DivisionByZero`;
  - `new(i128::MIN, 1)` and `from_int(i128::MIN)` → `Overflow`;
  - 1/3 + 1/6 = 1/2; 146097/400 − 365 = 97/400; 2/3 × 3/4 = 1/2; (1/2) ÷ (1/4) = 2;
  - `checked_add(i128::MAX/1, 1/1)` → `Overflow`;
  - multiplying `(i128::MAX/2)` by `(2/i128::MAX)` gives 1, which only works if common factors are cancelled first (research.md R3);
  - `floor` and `rem_floor` for 7/2 (3, 1/2), −7/2 (−4, 1/2) and −4 (−4, 0);
  - `round` in all three directions for 5/2, −5/2, 7/2 and 1/3 (NearestEven: 5/2 → 2, −5/2 → −2, 7/2 → 4);
  - ordering: −1/2 < 1/3 < 1/2, including values near `i128::MAX`;
  - `Display`: "1/2", "3", "-7/2".

### Implementation

- [x] T006 [US2] Implement `Ratio` in `crates/uni-time/src/ratio.rs` per the contract and data-model.md: fields `num` and `den`, kept in lowest terms with `den > 0` and `num != i128::MIN`; `checked_*` operations cancel common factors before multiplying (R3); `Ord` compares by value without overflow (cross-multiplying through a 256-bit product built from two `u128` halves, or an equivalent exact method). Work until T005 passes.

**Checkpoint**: Story 2 works on its own.

---

## Phase 4: User Story 1 - Exact points in time (Priority: P1)

**Independent Test**: `cargo test -p uni-time instant span`.

### Tests (write first; they must fail)

- [x] T007 [P] [US1] In `crates/uni-time/src/span.rs`, write unit tests:
  - `from_nanos` at ±(2⁶⁴×10⁹ − 1) succeeds and ±2⁶⁴×10⁹ fails with `OutOfRange`;
  - `from_secs` and `as_nanos`;
  - add, sub and negate, including results out of range;
  - `checked_mul_int`, and `checked_div_int` with each `Rounding` for 7 ns ÷ 2 (Down 3, Up 4, NearestEven 4) and −7 ns ÷ 2 (−4, −3, −4);
  - division by 0 → `DivisionByZero`;
  - ordering.
- [x] T008 [P] [US1] In `crates/uni-time/src/instant.rs`, write unit tests:
  - `new(0, 1_000_000_000)` → `OutOfRange`;
  - `from_nanos(-500_000_000)` gives `secs −1, nanos 500_000_000`;
  - `as_nanos` round-trips;
  - ordering across zero and 1 ns apart;
  - at ±100,000 years (±3_155_695_200_000 s), add 1 ns and `since` gives exactly 1 ns;
  - add then subtract a span returns the original;
  - `new(i64::MAX, 999_999_999)` + 1 ns → `OutOfRange`, and the same at `i64::MIN` − 1 ns;
  - `since` between the extreme instants doesn't overflow.

### Implementation

- [x] T009 [US1] Implement `Span` in `crates/uni-time/src/span.rs` per the contract (the US1 methods only: construction, `as_nanos`, add, sub, negate, integer multiply and divide). Keep the range |nanoseconds| < 2⁶⁴ × 10⁹ (data-model.md). Work until T007 passes.
- [x] T010 [US1] Implement `Instant` in `crates/uni-time/src/instant.rs` per the contract and data-model.md, converting through `as_nanos`/`from_nanos` (i128 nanoseconds; R1). Work until T008 passes.

**Checkpoint**: Story 1 works on its own.

---

## Phase 5: User Story 3 - Moving between moments and fractions (Priority: P2)

**Independent Test**: `cargo test -p uni-time ratio_conversions`.

### Tests (write first; they must fail)

- [x] T011 [US3] In `crates/uni-time/src/span.rs`, add a `ratio_conversions` test module:
  - `as_secs_ratio` of 1.5 s gives 3/2, and of −1 ns gives −1/1_000_000_000;
  - round trip span → ratio → span (any rounding) is exact for whole nanoseconds;
  - `from_secs_ratio(1/3, Down/Up/NearestEven)` gives 333_333_333 / 333_333_334 / 333_333_333 ns;
  - the tie 5/2_000_000_000 with NearestEven gives 2 ns, and the tie 7/2_000_000_000 gives 4 ns;
  - negative thirds round towards −∞ with Down;
  - `checked_mul_ratio(1 s, 1/7, Down)` gives 142_857_142 ns;
  - an out-of-range result → `OutOfRange`, never a clipped value.

### Implementation

- [x] T012 [US3] Implement `as_secs_ratio`, `from_secs_ratio` and `checked_mul_ratio` in `crates/uni-time/src/span.rs`, using `Ratio`'s exact arithmetic and `round`. Work until T011 passes.

---

## Phase 6: Independent oracle and performance

- [x] T013 [P] Write `crates/uni-time/tests/properties.rs` with `proptest` at 10,000 cases per property (`ProptestConfig::with_cases(10_000)`). Each `Ratio` operation (add, sub, mul, div, cmp, floor, rem_floor, round in all three directions) is compared with `num_rational::BigRational`: where the exact result fits in our range, ours must equal the oracle's; where it doesn't, ours must return `Overflow`. Also check:
  - instant + span − span = instant;
  - `a.since(b) + b = a` (via `checked_add`);
  - span → ratio → span round trips.
  
  Inputs include values near the range limits.
- [x] T014 [P] Write `crates/uni-time/tests/timing.rs`, a `#[test] #[ignore]` that times a million mixed operations (`Instant::checked_add`, `since`, `Ratio::checked_add`/`checked_mul`, `Span::from_secs_ratio`) with `std::time::Instant` and asserts the elapsed `Duration` is under one second. No floats (R7).
- [x] T015 Run `cargo test -p uni-time --release -- --ignored` and record the timing in the build PR description (SC-005).

---

## Phase 7: Polish

- [x] T016 Remove "the frame chain (GCT, SBT, proper time) and clock tables" from the crate doc in `crates/uni-time/src/lib.rs` if it isn't already gone, noting those arrive in later phases. Add doc comments with a short example to each public type.
- [x] T017 Run the full local verification and record the results in the build PR description:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `cargo deny check advisories bans sources`
  - `python scripts/check_architecture.py`
  - `python -m unittest discover -s scripts/tests`
  - `python scripts/check_okf.py knowledge`
  
  Also confirm `uni-time` is caught by the architecture check: temporarily add `let x: f64 = 1.0;` and revert.
- [ ] T018 Open the build PR as the bot: label `build`, body containing `Design: #<design PR>` and `Closes #10`. Move card #10 to **PR review**.

---

## Dependencies & Execution Order

- **Setup → Foundational**: T001 → T002 → T003 and T004 (in parallel).
- **US2 (fractions)** needs the foundational phase; it comes first because US3 needs it.
- **US1 (instants and spans)** needs the foundational phase only, and can run alongside US2.
- **US3** needs US1 and US2.
- **Phase 6** needs every story. T013 and T014 can run in parallel.
- **Polish** comes last.

Within each story the order is tests, then implementation, then the checkpoint.

## Implementation Strategy

Fractions (US2) and instants (US1) are each usable on their own; US3 joins them. Everything ships in one build PR, which fits a 30-minute review (about 600 lines).

## Notes

- Tick each task in `tasks.md` as it's done; the PR policy refuses a build PR with unticked tasks.
- Build PRs may only change `tasks.md` within `specs/`. A design change discovered during the build needs a new design PR.
