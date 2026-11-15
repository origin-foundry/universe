---
description: "Task list for spec 005, expression language basics"
---

# Tasks: Expression Language Basics

**Input**: Design documents from `specs/005-expression-language/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/uni-expr-api.md](contracts/uni-expr-api.md), [quickstart.md](quickstart.md)

**Tests**: Included, and written first (constitution VI, FR-012). Each test task must fail before its implementation task.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [ ] T001 In `crates/uni-expr/Cargo.toml`, add `publish = false` and `[dev-dependencies]` `proptest` and `num-bigint` at exactly the versions in `crates/uni-calendar/Cargo.toml`. Add no runtime dependencies (research R7).
- [ ] T002 Replace `crates/uni-expr/src/lib.rs` with: a crate doc comment (the first subset of the expression language: whole numbers and true/false, checked before use, exact evaluation, periods; no floating point, constitution III); keep `#![deny(clippy::float_arithmetic)]`; `mod types; mod error; mod lex; mod parse; mod check; mod arith; mod eval; mod period;`; `pub use` of every public type in the contract, and `pub const MAX_DEPTH: usize = 256;`. Create the module files with the contract's types and method signatures, each method body `todo!()`, so the crate builds and tests can be written against it (and fail). Every `todo!()` is gone by T016; the finished language never panics.

---

## Phase 2: Foundational

**Purpose**: the value, type, position and error types every story uses, and the floored arithmetic.

- [ ] T003 [P] Write `crates/uni-expr/src/types.rs`: `Type { Int, Bool }` with `Display` "whole number" / "true/false"; `Value { Int(i64), Bool(bool) }` with a `ty()` method; `VarDecl { name, ty }`; `Pos { line, column }` ("both from 1; columns in chars") with `Display` `line:column`. Derives as in the contract.
- [ ] T004 [P] Write `crates/uni-expr/src/error.rs`: `CheckError`, `EvalError` and `PeriodError` exactly as in the contract, deriving `Clone, Debug, PartialEq, Eq`, implementing `Display` with the messages in data-model.md (for example "`+` needs a whole number, found true/false at 1:8", "division by zero at 1:5", "`/` at 1:6 stops the expression repeating as `year` counts up") and `std::error::Error`. Include unit tests of one message per variant.
- [ ] T005 [P] Write `crates/uni-expr/src/arith.rs` test-first: checked floored `div(a, b)` and `rem(a, b)` per research R5, returning `None` for `b = 0` and for `i64::MIN / -1`, plus checked `add`, `sub`, `mul`, `neg`. Unit tests: `div(-7, 4) = -2`, `rem(-7, 4) = 1`, `div(7, -4) = -2`, `rem(7, -4) = -1`, `div(-7, -4) = 1`, `rem(-7, -4) = -3`, `rem(0, 5) = 0`, `div(i64::MIN, -1) = None`, `rem(5, 0) = None`, and for many `a`, `b`: `div(a, b) * b + rem(a, b) = a` with `rem` having the sign of `b` or being 0. Do not use `div_euclid` or `rem_euclid`.

**Checkpoint**: `cargo build -p uni-expr --all-targets` and `cargo clippy -p uni-expr --all-targets -- -D warnings` pass.

---

## Phase 3: User Story 1 - Check an expression (Priority: P1) 🎯 MVP

**Goal**: text is accepted as a checked `Expr` or refused with the reason and line and column.

**Independent Test**: `cargo test -p uni-expr --test check`, `cargo test -p uni-expr lex::`, `parse::` and `check::`.

### Tests (write first; they must fail)

- [ ] T006 [P] [US1] Write `crates/uni-expr/tests/check.rs` covering Story 1's acceptance scenarios and quickstart.md's table, each asserting the exact `CheckError` variant and `Pos`, with `year` declared as `Type::Int`:
  - the Aster rule and the Gregorian rule with a `Bool` result are accepted, and `source()` returns the text unchanged; `if year % 2 == 0 then year / 2 % 3 == 0 else false` is accepted;
  - `year % 4` (Bool expected) → `TypeMismatch { context: "result", expected: Bool, found: Int, at: 1:1 }`;
  - `year % 4 == 0 &&` → `Unexpected { found: None, expected: "a value", at: 1:17 }`;
  - `month == 2` → `UnknownVariable { name: "month", at: 1:1 }`;
  - `year + true` → `TypeMismatch` with context "+" at 1:8; `!year` → context "!" at 1:2; `if year then 1 else 2` → context "if" at 1:4; `if true then 1 else false` → context "then/else" at the `false`; `1 == true` → context "==";
  - a two-line expression `year % 4 == 0 &&\n  year + true` → the fault at line 2, column 10;
  - `1 < year < 5` → `ChainedComparison { at: 1:10 }`;
  - `99999999999999999999` (Int expected) → `LiteralOutOfRange { at: 1:1 }`; `-9223372036854775808` is accepted (research R2); `9223372036854775808` is refused;
  - `year # 2` → `InvalidCharacter { found: '#', at: 1:6 }`; `(year` → `Unexpected` at the end; `year)` → `Unexpected { found: Some(")") }`;
  - a variable named `année` is accepted and positions after it count characters, not bytes;
  - declarations: `if` as a name, `2x`, an empty name, and two `year`s → `InvalidVariable` with the reason;
  - 257 nested parentheses and a chain of 300 `+ 1` terms → `TooDeep { limit: 256, .. }`; 200 nested parentheses are accepted.
- [ ] T007 [P] [US1] Write unit tests in `crates/uni-expr/src/lex.rs` (tokens and their positions for every operator, keyword, name and literal; whitespace including `\r\n` and tabs; positions on later lines), `crates/uni-expr/src/parse.rs` (precedence and grouping: `1 + 2 * 3`, `a - b - c` groups left, `-a * b`, `!a && b`, `a || b && c`, `if c then 1 else 2 + 3` takes `2 + 3` as the else branch) and `crates/uni-expr/src/check.rs` (types of each node kind; variables resolved to their declaration index).

### Implementation

- [ ] T008 [US1] Implement the lexer in `crates/uni-expr/src/lex.rs` per research R2: decimal literals (kept as text until the parser folds a leading `-`), names with `char::is_alphabetic` / `is_alphanumeric` / `_`, keywords, operators (`|| && == != <= >= < > + - * / % ! ( )`), whitespace by `char::is_whitespace`, positions in characters from 1:1; anything else is `InvalidCharacter`.
- [ ] T009 [US1] Implement the parser in `crates/uni-expr/src/parse.rs` per the contract's grammar: recursive descent, left-associative loops for `||`, `&&`, `+ -`, `* / %`; at most one comparison per `cmp` (a second is `ChainedComparison`); `if` as an atom; a unary `-` directly before a literal folded into it; literals outside `i64` refused; every node with its start position and operator position; the tree depth and the parser's own recursion counted against `MAX_DEPTH` (research R4), refusing with `TooDeep`.
- [ ] T010 [US1] Implement `Expr::check`, `source`, `ty` and `vars` in `crates/uni-expr/src/check.rs` per research R3: check declarations first ("A valid name … not a keyword, unique among the declarations", data-model.md); resolve names to indices; type each node by the rules in R3, reporting `TypeMismatch` at the start of the offending operand; require the requested result type (context "result" at 1:1). Work until T006 and T007 pass.

**Checkpoint**: Story 1 works on its own.

---

## Phase 4: User Story 2 - Evaluate an expression (Priority: P1)

**Goal**: a checked expression evaluates exactly, or reports division by zero or out of range at the operator.

**Independent Test**: `cargo test -p uni-expr --test eval`.

### Tests (write first; they must fail)

- [ ] T011 [US2] Write `crates/uni-expr/tests/eval.rs` covering Story 2's acceptance scenarios:
  - the Aster rule at 5, 100, 500, 0, −5, 1 → true, false, true, true, true, false; the Gregorian rule at 2000, 1900, 2024, 2023, 0, −4, −100 → true, false, true, false, true, true, false;
  - `-7 % 4`, `-7 / 4`, `7 % -4`, `7 / -4` → 1, −2, −1, −2;
  - `if year % 2 == 0 then year / 2 % 3 == 0 else false` at 6, 4, 3 → true, false, false;
  - `100 / (year % 7) == 3` at 14 → `DivisionByZero { at: 1:5 }`; at 1 → false; `year % 0` → `DivisionByZero`;
  - `year * 4611686018427387904` at 2 → `OutOfRange { at: 1:6 }`; `-year` at `i64::MIN` → `OutOfRange`; `year / -1` at `i64::MIN` → `OutOfRange`;
  - `year % 2 == 0 && 100 / (year % 7) == 3` at 7 → false with no error; `year % 2 == 1 || 1 / 0 == 0` at 7 → true; `if true then 1 else 1 / 0` → 1;
  - two variables (`year: Int`, `leap: Bool`): values are taken in declaration order; passing one value, three values, or a `Bool` for `year` → `Inputs`, never a panic.

### Implementation

- [ ] T012 [US2] Implement `Expr::eval` in `crates/uni-expr/src/eval.rs`: check the inputs against the declarations (`Inputs`), then walk the tree using `arith.rs` for every whole-number operation, mapping `None` to `DivisionByZero` (divisor 0) or `OutOfRange` at the operator's position; `&&`, `||` and `if` evaluate only what they need (FR-007). Work until T011 passes.

**Checkpoint**: Stories 1 and 2 work together; the language is usable.

---

## Phase 5: User Story 3 - Work out how an expression repeats (Priority: P1)

**Goal**: a checked expression gives a safe period in one whole-number variable, or the part that stops it repeating.

**Independent Test**: `cargo test -p uni-expr --test period` and `cargo test -p uni-expr period::`.

### Tests (write first; they must fail)

- [ ] T013 [P] [US3] Write `crates/uni-expr/tests/period.rs` covering Story 3's acceptance scenarios (limit 100,000 unless stated):
  - the Aster rule → 500; the Gregorian rule → 400; `(11 * year + 14) % 30 < 11` → 30; `true` and `3 > 2` → 1; `year % 4 == 0 || year % 6 == 0` → 12; `(year + 3) % 4 == 0` → 4; `(-2 * year) % 6 == 0` → 6; `year % -4 == 0` → 4;
  - `year / 7 % 2 == 0`, `year * year % 3 == 0`, `year > 100` → `NotRepeating { at: 1:6 }`; `year % (year % 3 + 2) == 0` → `NotRepeating` at the outer `%`; `(if year % 2 == 0 then year else 0) % 4 == 0` → `NotRepeating` at the `if`;
  - `year % 7 == 0 && year % 11 == 0 && year % 13 == 0` with limit 1,000 → `OverLimit { limit: 1000 }`, and with limit 1,001 → 1,001;
  - a second variable `n` (Int) is constant: `year % 4 == n` → 4, while `year % n == 0` → `NotRepeating`;
  - `period("month", …)` and `period` of a `Bool` variable → `NotAWholeNumber`; limit 0 → `BadLimit`;
  - for each accepted expression above, `eval` at y and y + P agree for every y from −2P to 2P (Story 3 scenario 7).
- [ ] T014 [P] [US3] Write unit tests in `crates/uni-expr/src/period.rs` for each form rule in research R6: `Const` folding (including a failing fold such as `1 / 0` becoming `Periodic(1)`), `Affine` combination and its coefficient overflow (`year * 4611686018427387904 * 4` → `NotRepeating`), `Affine % m` → `Periodic(|m|)`, `Affine % 0` → `Periodic(1)`, lcm of periodic parts, and the early `OverLimit`.

### Implementation

- [ ] T015 [US3] Implement `Expr::period` in `crates/uni-expr/src/period.rs` per research R6: validate the variable and limit; one bottom-up pass assigning `Const(c)`, `Affine(a, b)`, `Periodic(P)` or `NotRepeating(at)` by the rules in R6, with `lcm` computed by checked arithmetic and refused as soon as it passes the limit; return the root's period (`Const` → 1). Work until T013 and T014 pass.

**Checkpoint**: Stories 1 to 3 work; leap rules (spec 006) can build on this.

---

## Phase 6: User Story 4 - Exact and safe on any input (Priority: P2)

**Goal**: property tests prove evaluation exact, periods safe, and checking crash-free on any text.

**Independent Test**: `cargo test -p uni-expr --test properties`.

### Tests

- [ ] T016 [US4] Write `crates/uni-expr/tests/properties.rs` with `proptest` at 10,000 cases per property (`ProptestConfig::with_cases(10_000)`), per research R8. A strategy builds random well-typed expression trees (depth up to 8; literals from −1,000 to 1,000 plus a few near `i64::MAX`; variables `year: Int` and `n: Int`; every operator; `if`), rendered as text both fully parenthesised and with the fewest parentheses precedence allows. Properties:
  - both renderings check, and evaluating them for random `year` and `n` gives the same result as a reference evaluator in the test file that walks the generated tree with `num_bigint::BigInt`, implements floored `/` and `%` by its own definition, honours short-circuiting, and reports out of range if any value it computes doesn't fit `i64` and division by zero where it meets one (Story 4 scenario 1);
  - for trees whose period in `year` is worked out (limit 10,000), the reference gives the same value or the same division by zero at `y` and `y + P` for random `y` from −10⁶ to 10⁶ wherever it reports no out of range (Story 4 scenario 2);
  - random strings (any characters, up to 2,000 long) and generated deep nestings (`(` × k, `- ` × k, `1 + ` × k, for k up to 5,000) are checked without panicking, and every error position lies within the text or one past its end (Story 4 scenario 3).
- [ ] T017 [US4] Fix anything T016 finds in `crates/uni-expr/src/`, adding each failing case as a regression test in the matching unit-test module (constitution VI). If nothing fails, record that in the build PR description.

---

## Phase 7: Performance and polish

- [ ] T018 [P] Write `crates/uni-expr/tests/timing.rs`: a `#[test] #[ignore]` that checks the Gregorian rule once and evaluates it 1,000,000 times for years from −500,000 to 499,999, timing with `std::time::Instant` and asserting under one second (under 1 µs each). No floats.
- [ ] T019 Run `cargo test -p uni-expr --release -- --ignored` and record the timing in the build PR description (SC-004).
- [ ] T020 Add doc comments with a short example (the contract's example) to `Expr::check`, `Expr::eval` and `Expr::period`, and finish the crate doc in `crates/uni-expr/src/lib.rs`: what this subset covers, and what comes later (`let`, functions, other types, sky functions, storing the AST).
- [ ] T021 Run the full local verification and record the results in the build PR description:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `cargo deny check advisories bans sources`
  - `python scripts/check_architecture.py`
  - `python -m unittest discover -s scripts/tests`
  - `python scripts/check_okf.py knowledge`
  - `python scripts/check_privacy.py`

  Also confirm the architecture check catches floating point in `uni-expr`: temporarily add `let x: f64 = 1.0;` to `crates/uni-expr/src/lib.rs`, see the check fail, and revert (SC-005).
- [ ] T022 Open the build PR as the bot from `build/005-expression-language`: label `build`, body with `Design: #<design PR>` and `Closes #44`. Move card #44 to **PR review** with `python scripts/board.py move 44 "PR review"`.

---

## Dependencies & Execution Order

- **Setup → Foundational**: T001 → T002 → T003, T004 and T005 (in parallel).
- **US1** (T006 to T010) needs the foundational phase. Every other story needs a checked `Expr`, so US1 comes first. T006 and T007 can be written in parallel; T008 → T009 → T010.
- **US2** (T011, T012) needs US1.
- **US3** (T013 to T015) needs US1, and uses US2's `eval` in T013's last check. T013 and T014 can be written in parallel.
- **US4** (T016, T017) needs US2 and US3.
- **Performance and polish** come last; T018 can be written alongside US4.

Within each story the order is tests, then implementation, then the checkpoint.

## Parallel examples

- After T002: T003 (types), T004 (errors) and T005 (arithmetic) touch different files.
- In US1: T006 (`tests/check.rs`) and T007 (unit tests in `src/`).
- In US3: T013 (`tests/period.rs`) and T014 (`src/period.rs` tests).

## Implementation Strategy

US1 alone (checking) is the MVP slice; US2 makes the language usable; US3 gives leap rules what they need; US4 proves it exact and safe everywhere. Everything ships in one build PR of about 1,100 lines, close to half of them tests (plan.md, principle VIII).

## Notes

- Tick each task in `tasks.md` as it's done; the PR policy refuses a build PR with unticked tasks.
- Build PRs may only change `tasks.md` within `specs/`. A design change discovered during the build needs a new design PR.
- Spec 006 (leap rules, #43) starts its design PR after this feature's build PR merges; it carries spec 004's range-wording correction.
