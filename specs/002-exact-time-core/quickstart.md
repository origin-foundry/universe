# Quickstart: validating the Exact Time Core

From the repository root.

1. `cargo test -p uni-time` runs the known-answer tests and the property tests (10,000 cases each; SC-001, SC-002, SC-003). Everything passes.
2. `cargo test -p uni-time -- --ignored` runs the timing check: a million mixed operations in under a second (SC-005). Run it on the owner's machine in release mode: `cargo test -p uni-time --release -- --ignored`.
3. `python scripts/check_architecture.py` passes with `uni-time` in the `exact` list (SC-004). Adding `let x: f64 = 1.0;` to `crates/uni-time/src/lib.rs` makes it fail; revert afterwards.
4. Spot checks the tests must include:

   | Case | Expected |
   |---|---|
   | `Ratio(1,3) + Ratio(1,6)` | `1/2` |
   | `Ratio(2,4)` | stored as `1/2` |
   | `Ratio(-7,2).floor()`, `.rem_floor()` | `-4`, `1/2` |
   | `Ratio::new(1, 0)` | `DivisionByZero` |
   | `Ratio(i128::MAX,1) + Ratio(1,1)` | `Overflow` |
   | `Instant::new(-1, 500_000_000)` vs `Instant::new(0, 0)` | earlier |
   | ±100,000 years, + 1 ns, then `since` | exactly 1 ns |
   | `Instant::new(i64::MAX, 999_999_999) + 1 ns` | `OutOfRange` |
   | `Span::from_secs_ratio(1/3 s, Down / Up / NearestEven)` | 333,333,333 / 333,333,334 / 333,333,333 ns |
   | `Span::from_secs_ratio(5/2 000 000 000 s, NearestEven)` | 2 ns (a tie, which rounds to the even value) |
