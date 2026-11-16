# Implementation Plan: Expression Language Basics

**Branch**: `design/005-expression-language` (design), `build/005-expression-language` (build) | **Date**: 2026-11-15 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/005-expression-language/spec.md` (approved by the owner, 2026-11-15)

## Summary

`uni-expr` gains its first language: a hand-written lexer and recursive-descent parser for whole numbers, true/false values, arithmetic with floored `/` and `%`, comparisons, `&& || !` and `if … then … else`, a type checker that resolves the caller's declared variables, an exact evaluator with checked `i64` arithmetic and short-circuiting, and a one-pass analysis that works out a safe period in one whole-number variable (or says which part stops it repeating). Every failure is a structured error with a line and column. Details are in [research.md](research.md).

## Technical Context

**Language/Version**: Rust 1.99.1 (pinned), edition 2024.

**Primary Dependencies**: none at run time. Dev-only: `proptest` and `num-bigint` (the reference evaluator, research R8), at the versions `uni-calendar` already uses.

**Storage**: N/A (engine only; spec Assumptions).

**Testing**: `cargo test -p uni-expr`: hand-worked tests for every acceptance scenario, property tests (10,000 cases each) against a `BigInt` reference evaluator, for periods, and for arbitrary text, and a timing check run by hand (research R8).

**Target Platform**: any; tested on Windows and Linux in CI.

**Project Type**: library crate.

**Performance Goals**: under 1 µs to evaluate a leap-rule-sized expression (SC-004).

**Constraints**: no floating point (exact crate); no panics on any input (FR-011); tree depth at most 256 (research R4).

**Scale/Scope**: one crate, about 1,100 lines: roughly 650 of language code and 450 of tests. One build PR; see principle VIII below.

## Constitution Check

*GATE: passed before research and again after design.*

| Principle | How this plan complies |
|---|---|
| I. Spec before code | Approved spec; this design PR, then a build PR. |
| II. Knowledge is the source of truth | Follows `/language/expression-language.md` (typed, side-effect-free, compiled to an AST, type-checked on save, floored `%`) and `/architecture/crate-layout.md`. This PR updates the expression-language concept with the first subset and repetition, back to draft for the owner to re-verify. |
| III. Exact calendar arithmetic | Checked `i64` arithmetic only; `uni-expr` is already an exact crate with `#![deny(clippy::float_arithmetic)]`. |
| IV. Hidden absolute timeline | Not touched: the language has no instants yet. |
| V. Layering | `uni-expr` gets no dependencies on other crates; `/architecture/crate-layout.md` allows it `uni-time`, unused for now. No new crate or edge. |
| VI. Test-first; independent oracle | Tests are written before the code. Expected values are worked by hand; property tests compare with an independent `BigInt` reference evaluator written in the tests (research R8). No presets, so the preset oracle rule doesn't apply. |
| VII. No silent ambiguity | Every refusal names the reason and the line and column; arithmetic never wraps; a period that can't be shown is refused, never guessed. |
| VIII. Small changes | About 1,100 lines, close to spec 004's 1,000, which the owner reviewed in one sitting. This is the half left after the owner split the leap-rules spec for exactly this reason. No further split. |

## Project Structure

### Documentation (this feature)

```text
specs/005-expression-language/
├── spec.md, plan.md, research.md, data-model.md, quickstart.md
├── contracts/uni-expr-api.md
├── checklists/requirements.md
└── tasks.md
```

### Source Code

```text
crates/uni-expr/
├── Cargo.toml          # publish = false; no dependencies; dev: proptest, num-bigint
└── src/
    ├── lib.rs          # re-exports; #![deny(clippy::float_arithmetic)]
    ├── types.rs        # Type, Value, VarDecl, Pos
    ├── error.rs        # CheckError, EvalError, PeriodError
    ├── lex.rs          # tokens with positions + unit tests
    ├── parse.rs        # recursive descent, depth limit + unit tests
    ├── check.rs        # Expr::check: names, types + unit tests
    ├── arith.rs        # checked floored / and % + unit tests
    ├── eval.rs         # Expr::eval
    └── period.rs       # Expr::period: the four forms (research R6) + unit tests
crates/uni-expr/tests/
    ├── check.rs        # Story 1
    ├── eval.rs         # Story 2
    ├── period.rs       # Story 3
    ├── properties.rs   # Story 4: BigInt reference, periods, arbitrary text
    └── timing.rs       # #[ignore] performance check
```

**Changed in this design PR**: `knowledge/language/expression-language.md`, `knowledge/log.md`. The spec's Story 3 scenario 7 and FR-008 are tightened to say a period covers values and division by zero but not out-of-range errors (research R6), and its Edge Cases record the depth limit (R4).

## Complexity Tracking

No violations to justify.
