# Implementation Plan: Exact Time Core

**Branch**: `design/002-exact-time-core` (design), `build/002-exact-time-core` (build) | **Date**: 2026-11-02 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/002-exact-time-core/spec.md` (approved by the owner, 2026-11-02)

## Summary

`uni-time` gains three exact types: `Instant` (seconds plus nanoseconds), `Span` (signed nanoseconds) and `Ratio` (a 128-bit fraction in lowest terms). It also gains a caller-chosen `Rounding` and one `TimeError`. Every operation that can fail returns an error rather than wrapping, rounding or panicking. `uni-time` becomes an exact crate. Tests compare against an independent arbitrary-precision implementation. Details are in [research.md](research.md).

## Technical Context

**Language/Version**: Rust 1.99.1 (pinned), edition 2024.

**Primary Dependencies**: none at runtime. Dev-only: `proptest` (property tests), `num-rational` and `num-bigint` (the independent oracle, R4).

**Storage**: N/A.

**Testing**: `cargo test -p uni-time`: known-answer tests, property tests (10,000 cases each), and a timing check run by hand (R7).

**Target Platform**: any; tested on Windows and Linux in CI.

**Project Type**: library crate.

**Performance Goals**: under 1 µs per operation (SC-005).

**Constraints**: no floating point (exact crate); no panics on any input (every failure is a `TimeError`).

**Scale/Scope**: one crate, about 600 lines including tests.

## Constitution Check

*GATE: passed before research and again after design.*

| Principle | How this plan complies |
|---|---|
| I. Spec before code | Approved spec; this design PR, then a build PR. |
| II. Knowledge is the source of truth | Representation follows `/time/timeline-and-frames.md`. `/architecture/crate-layout.md` (`uni-time` becomes exact) and `/time/timeline-and-frames.md` (the float boundary lives in `uni-physics`) change, go back to draft, and need re-verification. |
| III. Exact calendar arithmetic | `uni-time` joins the exact crates. Whole numbers and exact fractions only, enforced by the architecture check and Clippy. |
| IV. Hidden absolute timeline | Not affected; nothing is shown to users. |
| V. Layering | `uni-time` stays at the bottom with no workspace dependencies. Its dev-dependencies are external. |
| VI. Test-first; independent oracle | Tests are written before the code. Expected values come from a separate big-integer implementation (R4), plus fixed known answers. |
| VII. No silent ambiguity | Every failure is a named error. Rounding is always chosen by the caller. |
| VIII. Small changes | One crate, about 600 lines; fits a 30-minute review. |

## Project Structure

### Documentation (this feature)

```text
specs/002-exact-time-core/
├── spec.md, plan.md, research.md, data-model.md, quickstart.md
├── contracts/uni-time-api.md
├── checklists/requirements.md
└── tasks.md
```

### Source Code

```text
crates/uni-time/
├── Cargo.toml          # dev-dependencies: proptest, num-rational, num-bigint
└── src/
    ├── lib.rs          # re-exports; #![deny(clippy::float_arithmetic)]
    ├── error.rs        # TimeError
    ├── rounding.rs     # Rounding
    ├── ratio.rs        # Ratio + unit tests
    ├── span.rs         # Span + unit tests
    └── instant.rs      # Instant + unit tests
crates/uni-time/tests/
    ├── properties.rs   # proptest against the BigRational oracle
    └── timing.rs       # #[ignore] performance check
```

**Changed in this design PR**: `knowledge/architecture/crate-layout.md` (`exact` gains `uni-time`), `knowledge/time/timeline-and-frames.md` (the float boundary's location), `knowledge/log.md`.

## Complexity Tracking

No violations to justify.
