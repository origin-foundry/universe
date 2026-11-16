# Quickstart: validating Leap Rules

From the repository root (on Windows, see the working notes in `CLAUDE.md` for the Rust `PATH` and build settings).

1. `cargo test -p uni-calendar` runs spec 004's tests unchanged in meaning (calendars without rules), the hand-worked leap-rule tests for Aster and Tessel, the property tests (10,000 cases each) against the year-by-year reference, and the Gregorian-shaped comparison with Python's `datetime` for years 1 to 4000. Everything passes (SC-001, SC-002, SC-003, SC-005).
2. To regenerate the Gregorian fixture: `python crates/uni-calendar/tests/data/gregorian_days.py > crates/uni-calendar/tests/data/gregorian_days.txt`. The output must not change (`git diff --exit-code`); its header names the Python version.
3. `cargo test -p uni-calendar --release -- --ignored` runs the timing checks: a conversion in a calendar with leap rules takes under 10 µs, near the epoch and 100,000 years away (SC-004). Run it on the owner's machine.
4. `python scripts/check_architecture.py` passes: `uni-calendar` depends only on `uni-time` and `uni-expr`, with no floating point (SC-006).
5. Spot checks the tests must include (Aster with its rule on Twelfth; values are year, month, sol, …):

   | Case | Expected |
   |---|---|
   | Check Aster with the rule | accepted; `rule_period()` 500; `units_per("year", "sol")` `NotFixed`; `units_per("year", "month")` 12 |
   | Condition `year % 4` | `RuleCondition { rule: 1, error: TypeMismatch { context: "result", .. } }` |
   | Condition `month == 2` | `RuleCondition { rule: 1, error: UnknownVariable { .. } }` |
   | Member "Thirteenth"; change 0 | `RuleUnknownMember`; `RuleZeroChange` |
   | Shorten Third by 55 when `year % 2 == 0` | `MemberTooShort { member: "Third", year: 2, length: 0 }` |
   | Condition `100 / (year % 7) == 3` | `RuleFails { year: 7, error: DivisionByZero { at: 1:5 } }` |
   | A rule on the decade | `RulesNotOnEraUnit { unit: "decade" }` |
   | Condition `year > 100` | `RulePeriod { error: NotRepeating { at: 1:6 } }` |
   | `[5, 12, 56]` → instant → back | same date; fixed number 3,340 |
   | `[1, 12, 56]` | `InvalidValue { unit: "sol", value: 56, first: 1, last: 55, within: Some("Twelfth"), year: Some(EraNumber { unit: "year", number: 1 }) }` |
   | `[6]`, `[501]` | fixed numbers 3,341 and 334,096 |
   | `[100, 12, 56]`; `[500, 12, 56]` | refused; accepted |
   | 1 ns before the epoch | year 0, Twelfth, sol 56, hour 19, minute 99, decade 0, age 0 |
   | `era_instance(5)` | Twelfth 56, length 669, holding `[1]` |
