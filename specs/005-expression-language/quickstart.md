# Quickstart: validating Expression Language Basics

From the repository root (on Windows, see the working notes in `CLAUDE.md` for the Rust `PATH` and build settings).

1. `cargo test -p uni-expr` runs the hand-worked tests for checking, evaluation and periods, and the property tests (10,000 cases each). Everything passes (SC-001, SC-002, SC-003).
2. `cargo test -p uni-expr --release -- --ignored` runs the timing check: evaluating the Gregorian rule takes under 1 µs (SC-004). Run it on the owner's machine.
3. `python scripts/check_architecture.py` passes: `uni-expr` has no dependencies on other crates and no floating point (SC-005). Adding `let x: f64 = 1.0;` to `crates/uni-expr/src/lib.rs` makes it fail; revert afterwards.
4. Spot checks the tests must include (`year` is a whole-number variable; "bool" means a true/false result is required):

   | Case | Expected |
   |---|---|
   | Aster rule, bool, eval at 5, 100, 500, 0, −5, 1 | true, false, true, true, true, false |
   | `year % 4`, bool | `TypeMismatch { context: "result", expected: Bool, found: Int, at: 1:1 }` |
   | `year % 4 == 0 &&` | `Unexpected { found: None, expected: "a value", at: 1:17 }` |
   | `month == 2` | `UnknownVariable { name: "month", at: 1:1 }` |
   | `year + true` | `TypeMismatch { context: "+", expected: Int, found: Bool, at: 1:8 }` |
   | `1 < year < 5` | `ChainedComparison { at: 1:10 }` |
   | `99999999999999999999` | `LiteralOutOfRange { at: 1:1 }` |
   | `-7 % 4`, `-7 / 4`, `7 % -4`, `7 / -4` | 1, −2, −1, −2 |
   | `100 / (year % 7) == 3` at 14 | `DivisionByZero { at: 1:5 }` |
   | `year * 4611686018427387904` at 2 | `OutOfRange { at: 1:6 }` |
   | `year % 2 == 0 && 100 / (year % 7) == 3` at 7 | false |
   | Periods: Aster rule, Gregorian rule, `(11 * year + 14) % 30 < 11`, `true` | 500, 400, 30, 1 |
   | Period of `year / 7 % 2 == 0`, `year * year % 3 == 0`, `year > 100` | `NotRepeating` at 1:6, 1:6, 1:6 |
   | Period of `year % 7 == 0 && year % 11 == 0 && year % 13 == 0`, limit 1,000 | `OverLimit { limit: 1000 }` |
