# Research: Calendar Engine Basics

## R1. How a definition is shaped

- **Decision**: a calendar definition is a plain Rust value (`CalendarDef`) that the engine checks once and turns into a ready-to-use `Calendar`. Its units form one chain, top to bottom:
  - **groupings** above the fundamental unit, listed top to bottom (age, decade, year);
  - the **fundamental unit** (sol), with an exact length;
  - **subdivisions** below it, listed top to bottom (hour, minute).
- **Members and the member unit**: a grouping lists its members in order; each member is a run of a whole number of the unit directly below the grouping in the chain, with an optional name (FR-003). When a grouping has more than one member, it names its **member unit**, the unit its members are instances of. Aster's year is a grouping over sols whose member unit is the month, with members Prime = 56 sols, Second = 56 sols, and so on. A decade is a grouping over years with one member of 10 years and no member unit. This is exactly the shape of the verified [Gregorian example](../../knowledge/calendar/gregorian-example.md) (`year: kind grouping, children: month, members: [January: 31, …]`), so the month needs no definition of its own.
- **Rationale**:
  - **No redundancy:** each length is stated once. The alternative, where every unit including the month is its own grouping, has to say "12 months" in two places (the year and the decade) and check that they agree.
  - **Every grouping instance is the same length:** because member counts are fixed (no leap rules yet), every year is 668 sols and every decade 6,680. Only member-unit instances (months) vary, and they vary by position, not by year. R4 relies on this.
  - **Leap rules fit later:** leap rules (a later spec) adjust a grouping's member counts, the same place the Gregorian example puts them.
- **Validation added by this shape**: a grouping with more than one member and no member unit is refused (its members couldn't be told apart in a date). This is listed with the other refusals in data-model.md.
- **Alternatives**: a general unit tree with parent links (the storage schema's `unit.parent_id`). Branching trees are out of scope (spec, Assumptions), and a chain is simpler to check and to walk. The storage form is a later feature's job; it can map onto this chain.

## R2. Exact lengths

- **Decision**: the fundamental unit's length is a `uni_time::Ratio` of seconds (FR-002). Inside the engine it's converted once to nanoseconds as a reduced fraction `p/q` (`p`, `q` positive `i128`). Every unit boundary is then an exact fraction with denominator `q × P`, where `P` is the product of the subdivision part counts. All arithmetic is checked `i128` integer arithmetic: no floating point (FR-014, FR-017).
- **Rationale**: `Ratio` is the exact fraction type the time core already provides, so callers state lengths the same way everywhere. Working in integer "ticks" of `1/(q × P)` ns keeps every step exact and makes overflow an explicit, checked condition.
- **No change to `uni-time`**: the engine needs only `Instant::as_nanos`/`from_nanos`, `Ratio` and its checked operations, which spec 002 already provides.

## R3. The engine's limits

The supported range (FR-015) has to be stated precisely so that a valid calendar never fails part-way through the range it promises. A definition is refused, naming the unit and the limit, when:

| Limit | Why |
|---|---|
| The fundamental unit's length in nanoseconds, as a reduced fraction, has a denominator over 10⁹. | Keeps `instant offset × q` inside `i128` across the whole `Instant` range (offsets are under 1.9 × 10²⁸ ns). |
| The smallest unit (the finest subdivision, or the fundamental unit if there are none) is shorter than 1 ns. | Date-to-instant returns the first whole nanosecond at or after a date's start (FR-014). That nanosecond is less than 1 ns after the start, so it stays inside the same date only if every date lasts at least 1 ns. Without this, round trips (US4) would fail. |
| The fundamental unit's length, counted in ticks of `1/(q × P)` ns, is 2¹²⁰ or more. | Keeps sub-unit arithmetic inside `i128`. A million-year sol with nanosecond-exact length still allows 2⁴⁵ smallest units per sol. |
| A grouping's length in fundamental units doesn't fit in `i64`. | Fixed numbers are `i64` (R5). |

The spec's examples all sit well inside these: a 1 ns fundamental unit and a million-year one are both accepted (Edge Cases). The limits are added to the spec's Edge Cases so the owner sees them.

## R4. Conversion algorithm (FR-012, FR-016)

- **Decision**:
  1. **Instant to fixed number**: `offset = instant − epoch` in nanoseconds (`i128`; always fits); `fixed = ⌊offset × q / p⌋`; the position inside the fundamental unit is the remainder `offset × q − fixed × p`, in `[0, p)`, in units of `1/q` ns.
  2. **Units above the fundamental unit**: walk the groupings top to bottom. The top-level index is `⌊fixed / L_top⌋` (floor division, so negative fixed numbers work); each lower grouping's index within its parent is the offset divided by its length; a member unit's index is found in the grouping's prefix sums of member counts (binary search over at most a few hundred members).
  3. **Subdivisions**: for each subdivision into `n` parts, `index = ⌊r × n / p_level⌋` and the new remainder is `r × n − index × p_level`; the final remainder, divided by `q × P` ns, is the exact `Ratio` remainder below the smallest unit (FR-010).
  4. **Date to instant** reverses this, summing exact contributions, then rounds the exact start up to the next whole nanosecond, reporting the exact amount if it wasn't already whole (FR-014).
- **Rationale**: every grouping instance is the same length (R1), so the top-level unit is a repeating cycle and each level costs one division or one small binary search. That is the simplest form of the supercycle strategy in `/calendar/conversion.md`, constant time whatever the distance from the epoch (FR-016, SC-004). FR-016 names one instance of the era's unit as the cycle; with no leap rules every grouping instance is identical, so the era's unit and the top-level unit are both cycles, and walking from the top is the same computation.
- **Boundaries**: floor division puts an instant exactly on a boundary in the unit that starts there (spec, Edge Cases).

## R5. Numbers

- **Decision**: fixed numbers and every date value are `i64`. A conversion whose fixed number or any date value doesn't fit is `OutOfRange` (FR-015), as is a date whose start instant would leave the `Instant` range.
- **Numbering**:
  - **The era's unit and units above it** use astronomical numbering by count from the epoch: the instance that starts at the epoch is 1, the one before it 0, then −1 (FR-008a, FR-009). An instance's count from the epoch is `⌊fixed / length⌋` for groupings and the fundamental unit, and `parent count × members + member index` for a member unit, because every parent has the same members.
  - **Units below the era's unit** are numbered by position within their parent plus their first number (FR-005a). Defaults: 1 for member units, groupings and the fundamental unit; 0 for subdivisions.
  - A first number given for the era's unit or a unit above it is refused: the era numbers those.
- **Rationale**: `i64` covers ±100,000 years even with a 1 ns fundamental unit (3.2 × 10¹⁸ < 9.2 × 10¹⁸), and keeps dates small and `Copy`-friendly for later features.

## R6. Results and errors

- **Decision**:
  - `instant_to_date` returns an `InstantDate`: the numbers of the units above the era's unit, the full date from the era's unit to the smallest unit, and the exact remainder below it as a `Ratio` of seconds.
  - `date_to_instant` returns a `DateStart`: the `Instant`, and `rounded_by: Option<Ratio>`, the exact seconds added to reach a whole nanosecond (`None` when the start was already whole).
  - Two structured error enums (FR-019), implementing `Display` and `std::error::Error` by hand like `uni-time`:
    - `DefinitionError`, for refused definitions, each variant naming the unit (and member) at fault;
    - `CalendarError`, for refused conversions: `InvalidValue { unit, value, first, last, within }` (`within` names the member, such as "Third", when the range depends on it; FR-013), `OutOfRange`, `EmptyDate`, `TooManyValues`, `UnknownUnit`, `NotFixed`.
- **Rationale**: matching `uni-time`'s style keeps one way of reporting errors across the exact crates. `Option<Ratio>` makes "was it rounded?" impossible to ignore.
- **Alternatives**: a single error enum for both; rejected because a definition error is a bug in the calendar while a conversion error is a bad input, and later features show them in different places.

## R7. Tests and the independent check (constitution VI)

- **Decision**:
  - **Hand-worked dates** for Aster (every acceptance scenario in the spec) and for a second calendar of a different shape, **Tessel**: a fundamental unit "turn" of exactly 100,000 s split into 7 "watches" (so watch boundaries fall between nanoseconds, US4 scenario 3), and a "cycle" grouping over turns whose member unit is the "phase", with three unnamed members of 13, 13 and 14 turns. The era numbers cycles; turns count from 0 (a non-default first number) and phases from 1.
  - **Property tests** (`proptest`, 10,000 cases each; SC-002): random valid calendars (one to four groupings with up to 20 members, zero to three subdivisions, lengths with denominators up to 10⁹ in nanoseconds) and random instants within ±100,000 years of the epoch: instant → date → start + remainder gives back the instant exactly; date → instant → date gives back the date; a rounded start converts back to the same date.
  - **Independent oracle**: a deliberately simple reference implementation in the test code, using `num-rational`'s `BigRational` (arbitrary precision, so it never overflows), that finds a date by stepping through units one at a time. Property tests compare the engine with it over a smaller range (within ±50 top-level units of the epoch), where stepping is fast. It is a dev-dependency only, already in the lock file at the versions `uni-time` uses.
- **Rationale**: round trips alone can't catch a mistake made the same way in both directions (for example, numbering decades from 0); the oracle can.

## R8. Performance (SC-004)

- **Decision**: a `#[test] #[ignore]` timing check runs 100,000 conversions each way, half near the epoch and half 100,000 years away, and asserts each direction takes under one second (under 10 µs per conversion), using `std::time::Instant` and `Duration`. It's run by hand in release mode on the owner's machine, as in spec 002.

## R9. Crate wiring

- **Decision**: `uni-calendar` gains its first dependency, `uni-time` (a path dependency, allowed by `/architecture/crate-layout.md`), and `publish = false` (a path-only workspace dependency needs it, or `cargo deny` reports it as a wildcard). Dev-dependencies: `proptest`, `num-rational`, `num-bigint`, at the versions `uni-time` uses. No runtime dependencies beyond `uni-time`. The crate's existing `#![deny(clippy::float_arithmetic)]` stays, and it's already in the `exact` list.
- **No new crate or dependency edge** outside the allowed list, so no architecture change.

## R10. Knowledge changes in this design PR

- `/calendar/unit-kinds.md`: groupings' members and the member unit (R1), the per-unit first number (FR-005a), and the structure now "a chain under the top-level unit" for this phase with trees later.
- `/calendar/conversion.md`: the next-whole-nanosecond rule with the exact amount reported (FR-014); instant-to-date never rounds.
- `/calendar/eras-and-epoch.md`: an era numbers a chosen unit; units above it are numbered by count from the epoch (FR-001, FR-008a).
- New `knowledge/decisions/0008-physical-time-and-calendar-time.md` from the spec's ground rules, listed in `knowledge/decisions/index.md`.
- `knowledge/glossary.md`: **Cycle** becomes **Physical cycle**; the Calendar entry says "parallel cycles"; new terms **Physical time**, **Calendar time**, **Era's unit**, **First number**, **Member unit**, **Astronomical year numbering**.
- `knowledge/log.md` entry. Changed concepts go back to `status: draft` without `verified` until the owner re-verifies them.
