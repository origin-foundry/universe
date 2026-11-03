# Research: Exact Time Core

## R1. Types and their storage

- **Decision**:
  - `Instant { secs: i64, nanos: u32 }` with `nanos` in `0..1_000_000_000`. This is the verified design, and moments before zero have negative `secs`; for example, −0.5 s is `{ secs: -1, nanos: 500_000_000 }`.
  - `Span(i128)`: a signed count of nanoseconds.
  - `Ratio { num: i128, den: i128 }` with `den > 0` and `gcd(|num|, den) == 1`.
- **Rationale**:
  - **Spans as one number:** a single nanosecond count makes span arithmetic and comparison trivial, and keeps negative spans unambiguous.
  - **Spans can't overflow from subtraction:** the largest gap between two instants is about 1.8 × 10²⁸ ns, far inside `i128` (1.7 × 10³⁸).
  - **Instants keep the agreed shape:** `Instant` keeps the verified two-field layout, so storage (spec 003) sees seconds and nanoseconds directly.
- **Alternatives**:
  - `Span` as secs + nanos, like `Instant`: two fields to normalise for negative values, with no benefit.
  - `Instant` as a single `i128` count of nanoseconds: rejected because it contradicts the verified concept.

## R2. Range

- **Decision**: instants cover the whole `i64`-seconds range (about ±292 billion years). Spans are limited to strictly under 2⁶⁴ seconds either way (|nanoseconds| < 2⁶⁴ × 10⁹). The largest gap between two instants is (2⁶⁴ − 1) s + 999,999,999 ns, which is just inside that, so `since` can never fail. Arithmetic that leaves either range returns `TimeError::OutOfRange`.
- **Rationale**: this is far beyond FR-002's ±100,000 years, with no cost.

## R3. Write the fraction type ourselves

- **Decision**: hand-write `Ratio` in `uni-time`, with no runtime dependencies.
- **Rationale**:
  - **Exact errors:** we need checked arithmetic that reports overflow as `TimeError::Overflow` (FR-009).
  - **Our rounding:** we need the floor, remainder and rounding semantics in FR-008.
  - **No floats:** we need a type with no floating-point methods at all, so the crate can be exact (architecture check).
  
  It's about 200 lines. To avoid needless overflow, products and sums cancel common factors first: for multiplication, `gcd(a.num, b.den)` and `gcd(b.num, a.den)`; for addition, the least common multiple of the denominators.
- **Alternatives**: `num-rational`'s `Ratio<i128>`. Its plain operators panic or wrap on overflow, its checked operations don't cancel factors before multiplying, and it offers float conversions we'd have to avoid.

## R4. Independent test oracle (constitution VI)

- **Decision**: property tests use `proptest`. Expected answers come from `num-rational`'s `BigRational`, arbitrary precision, used as a **dev-dependency only**: an independent implementation that never overflows. Fixed known-answer tests (e.g. 1/3 + 1/6 = 1/2, Gregorian 146097/400) sit alongside.
- **Rationale**: comparing against a separate big-integer implementation catches mistakes that self-consistency checks (like "add then subtract") would miss. Dev-dependencies don't affect the shipped crate.
- **Note**: the architecture check only scans `uni-time`'s own source files. Neither test helper uses floats.

## R5. Errors

- **Decision**: one error enum, `TimeError { OutOfRange, DivisionByZero, Overflow }`, implementing `Display` and `std::error::Error` by hand, with messages like "time is out of range". Every fallible operation returns `Result<_, TimeError>`. Standard operators (`+`, `-`, `<`) are implemented only where they can't fail, which means comparisons. Arithmetic uses named `checked_*` methods, so a caller can't overlook an error.
- **Rationale**: constitution VII says ambiguity is never resolved silently. Operators that panic on overflow would turn data problems into crashes.
- **Alternatives**: the `thiserror` crate (one dependency for three variants); panicking operators.

## R6. Rounding

- **Decision**: `Rounding { Down, Up, NearestEven }`. Down means towards minus infinity and Up towards plus infinity, so behaviour is the same either side of zero. Conversions with a remainder finer than a nanosecond take a `Rounding` argument. There's no default.

## R7. Performance (SC-005)

- **Decision**: a `#[test] #[ignore]` timing check runs a million mixed operations and asserts they take under a second (under a microsecond each), using `std::time::Instant` and comparing `Duration`s, with no floats. It's run by hand (`cargo test -p uni-time -- --ignored`) in the build, not in CI, where timing is noisy.

## R8. Crate layout change

- **Decision**: this design PR adds `uni-time` to the `exact` list in `/architecture/crate-layout.md`. It also adds one sentence to `/time/timeline-and-frames.md` saying the floating-point boundary lives in `uni-physics` (Clarifications). Both go back to draft until the owner re-verifies them. The build then adds `#![deny(clippy::float_arithmetic)]` to `uni-time`, like the other exact crates.
