# Quickstart: validating the Calendar Engine Basics

From the repository root (on Windows, see the working notes in `CLAUDE.md` for the Rust `PATH` and build settings).

1. `cargo test -p uni-calendar` runs the hand-worked tests for Aster and Tessel, the refused definitions and dates, and the property tests (10,000 cases each). Everything passes (SC-001, SC-002, SC-003).
2. `cargo test -p uni-calendar --release -- --ignored` runs the timing check: conversions near the epoch and 100,000 years away each take under 10 µs (SC-004). Run it on the owner's machine.
3. `python scripts/check_architecture.py` passes: `uni-calendar` depends only on `uni-time` and has no floating point (SC-005). Adding `let x: f64 = 1.0;` to `crates/uni-calendar/src/lib.rs` makes it fail; revert afterwards.
4. Spot checks the tests must include (Aster is defined in [spec.md](spec.md); `E` is its epoch instant):

   | Case | Expected |
   |---|---|
   | `Calendar::new(aster)` | accepted; `units_per("year", "sol")` = 668, `units_per("sol", "minute")` = 2,000 |
   | `instant_to_date(E)` | age 1, decade 1; year 1, month 1, sol 1, hour 0, minute 0; remainder 0 |
   | `instant_to_date(E + 668 sols)` | year 2, month 1, sol 1 |
   | `instant_to_date(E − 1 ns)` | age 0, decade 0; year 0, month 12, sol 55, hour 19, minute 99; remainder 44.387621999 s |
   | Start of year 101 / year −100 | decade 11, age 2 / decade −10, age −1 |
   | `date_to_instant([1, 1, 1])` | `E`, `rounded_by: None` |
   | `date_to_instant([1, 3, 56])` | `InvalidValue` for sol: 56 outside 1 to 55, within Third |
   | `date_to_instant([1, 13])`, `[1, 1, 0]`, `[1, 1, 1, 20]`, `[1, 1, 1, 0, -1]` | `InvalidValue` naming month, sol, hour and minute with their ranges |
   | Aster with hours from 1: `instant_to_date(E)` | hour 1; `[1, 1, 1, 0]` refused as outside 1 to 20 |
   | Aster with a 100,000 s sol | accepted, no warning (FR-002a) |
   | Tessel ([research.md](research.md) R7; epoch `T`): `date_to_instant([1, 1, 0, 1])` (cycle 1, phase 1, turn 0, watch 1) | `instant` = `T + 14,285.714285715 s`; `rounded_by` = 5/7 ns (5/7,000,000,000 s), because the exact start is `T + 14,285,714,285,714 2/7 ns`; converting back gives watch 1 with remainder 5/7 ns |
