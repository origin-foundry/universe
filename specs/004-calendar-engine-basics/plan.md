# Implementation Plan: Calendar Engine Basics

**Branch**: `design/004-calendar-engine-basics` (design), `build/004-calendar-engine-basics` (build) | **Date**: 2026-11-10 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/004-calendar-engine-basics/spec.md` (approved by the owner, 2026-11-10)

## Summary

`uni-calendar` gains its first engine: a checked calendar definition (one fixed-length fundamental unit, a chain of fixed-count groupings above it with optional member units such as the month, a chain of equal subdivisions below it, and one era numbering a chosen unit with its epoch on the hidden timeline) and exact conversion both ways between instants and dates. All arithmetic is checked integer arithmetic over exact fractions, with no floating point. Invalid definitions and dates are refused with structured errors. The only rounding is to the next whole nanosecond when a date starts between two, and it's reported with the exact amount. Details are in [research.md](research.md).

## Technical Context

**Language/Version**: Rust 1.99.1 (pinned), edition 2024.

**Primary Dependencies**: `uni-time` (path dependency; `Instant`, `Ratio`). Dev-only: `proptest` (property tests), `num-rational` and `num-bigint` (the independent oracle, R7), at the versions `uni-time` already uses.

**Storage**: N/A (engine only; spec Clarifications).

**Testing**: `cargo test -p uni-calendar`: hand-worked tests for two calendars (Aster and Tessel), refusal tests, property tests (10,000 cases each) including a comparison with a `BigRational` reference implementation, and a timing check run by hand (R8).

**Target Platform**: any; tested on Windows and Linux in CI.

**Project Type**: library crate.

**Performance Goals**: under 10 µs per conversion, near the epoch or 100,000 years away (SC-004).

**Constraints**: no floating point (exact crate); no panics on any input; constant-time conversion (FR-016); engine limits stated and checked at definition time (R3).

**Scale/Scope**: one crate, about 1,000 lines: roughly 450 of engine code and 550 of tests. One build PR; see principle VIII below.

## Constitution Check

*GATE: passed before research and again after design.*

| Principle | How this plan complies |
|---|---|
| I. Spec before code | Approved spec; this design PR, then a build PR. |
| II. Knowledge is the source of truth | Follows `/calendar/calendar-model.md`, `/calendar/unit-kinds.md`, `/calendar/conversion.md`, `/calendar/eras-and-epoch.md` and the Gregorian example's grouping shape. This PR updates unit kinds (member units, first numbers), conversion (next-nanosecond rule), eras and epoch (the era's unit), adds ADR 0008 and glossary terms; each goes back to draft for the owner to re-verify. |
| III. Exact calendar arithmetic | Lengths are `Ratio`s; every boundary is an exact fraction; checked `i128` integer arithmetic only. `uni-calendar` is already an exact crate with `#![deny(clippy::float_arithmetic)]`. |
| IV. Hidden absolute timeline | The epoch is an `Instant` on the hidden timeline, set by later features' code, never shown. No UI. |
| V. Layering | `uni-calendar` → `uni-time` only, which `/architecture/crate-layout.md` allows. No new crate or edge. |
| VI. Test-first; independent oracle | Tests are written before the code. Expected dates are worked by hand; property tests check round trips and compare against an independent `BigRational` reference (R7). There are no presets yet, so the preset oracle rule doesn't apply. |
| VII. No silent ambiguity | Invalid definitions and dates are refused with the unit, value, range and reason; out-of-range results are errors; the one rounding (FR-014) is reported with its exact amount. |
| VIII. Small changes | About 1,000 lines, more than half of it tests that follow the spec's scenarios one to one. The engine code is one conversion path in each direction plus checking; that fits a 30-minute review. No split. |

## Project Structure

### Documentation (this feature)

```text
specs/004-calendar-engine-basics/
├── spec.md, plan.md, research.md, data-model.md, quickstart.md
├── contracts/uni-calendar-api.md
├── checklists/requirements.md
└── tasks.md
```

### Source Code

```text
crates/uni-calendar/
├── Cargo.toml          # publish = false; uni-time; dev: proptest, num-rational, num-bigint
└── src/
    ├── lib.rs          # re-exports; #![deny(clippy::float_arithmetic)]
    ├── error.rs        # DefinitionError, Limit, CalendarError
    ├── def.rs          # CalendarDef and its parts, Clock
    ├── calendar.rs     # Calendar::new (checking, flattening, precomputing) + unit tests
    ├── date.rs         # Date, InstantDate, DateStart
    └── convert.rs      # fixed numbers and both conversions + unit tests
crates/uni-calendar/tests/
    ├── common/mod.rs   # the Aster and Tessel definitions, shared by the test files
    ├── aster.rs        # hand-worked Aster dates (US1 to US3)
    ├── tessel.rs       # the second calendar, boundaries between nanoseconds
    ├── properties.rs   # round trips and the BigRational oracle (US4)
    └── timing.rs       # #[ignore] performance check
```

**Changed in this design PR**: `knowledge/calendar/unit-kinds.md`, `knowledge/calendar/conversion.md`, `knowledge/calendar/eras-and-epoch.md`, new `knowledge/decisions/0008-physical-time-and-calendar-time.md`, `knowledge/decisions/index.md`, `knowledge/glossary.md`, `knowledge/log.md`. The spec's Edge Cases gain the engine limits (R3).

## Complexity Tracking

No violations to justify.
