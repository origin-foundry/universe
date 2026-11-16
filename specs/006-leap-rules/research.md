# Research: Leap Rules

## R1. Where rules live in a definition

- **Decision**: `GroupingDef` gains `rules: Vec<LeapRuleDef>`. A `LeapRuleDef` is a `condition` (source text), the `member` it changes (by name, or by its number from 1 when the member has no name, as errors already label members) and a signed `change` (positive lengthens, negative shortens) in the unit the grouping's members count. Only the grouping that is the era's unit may have rules; rules anywhere else are refused (FR-005, Story 1 scenario 6).
- **Rationale**:
  - A rule belongs to the unit it changes, which is how `/calendar/leap-rules.md` describes it ("attached to a unit"), so the definition reads naturally and later features (rules on other groupings) need no reshaping.
  - A signed change is one field instead of a lengthen/shorten flag plus a count; zero is refused (Story 1 scenario 3).
  - Naming members by label reuses the label errors already use (`Unit::member_label`), so a member without a name is "1", "2", ….
- **Alternatives**: a calendar-level list of rules each naming its unit; rejected because it separates a rule from its unit and needs a lookup to find it.

## R2. What a condition sees, and checking it

- **Decision**: each condition is checked with `uni_expr::Expr::check(condition, &[VarDecl { name: <era unit's name>, ty: Int }], Type::Bool)`. The era's unit's name must therefore be a valid variable name (a letter or `_`, then letters, digits or `_`, not a keyword); if it isn't, every rule's check reports spec 005's `InvalidVariable`, which the calendar passes on (FR-005). The value is the astronomical number the date shows: the count of instances from the epoch plus 1 (spec 004), so year 0 and negative years are tested as written.
- **Rationale**: spec 005 already refuses everything FR-005 lists about a condition, with its position; the calendar only adds which rule it was.
- **Alternatives**: always calling the variable `year`; rejected because the spec (FR-002) and the definer's own naming ("cycle", "turn") ask for the unit's own name.

## R3. The rule period and its limit

- **Decision**: the rule period `P` is the lcm of every rule's period, each from `Expr::period(<era unit>, LIMIT)`, with **`LIMIT` = 100,000 instances**. A rule whose period can't be established is refused with spec 005's `PeriodError` (`NotRepeating` with its position, or `OverLimit`); rules whose combined lcm passes the limit are refused as a whole. A calendar without rules has `P = 1`.
- **Rationale**:
  - The spec asks for at least 10,000. The longest cycles of real arithmetic calendars are far below 100,000 (Gregorian 400, tabular Islamic 30, the 2,820-year arithmetic Persian cycle), and two unrelated rules can multiply periods, so 100,000 leaves room.
  - The cost of a long period is paid once, at checking: one evaluation per rule per year of the period (under 100 ns each, spec 005's timing), plus one table entry per year (R4). At 100,000 that's a few milliseconds and about 1.2 MB, which is acceptable for a checked calendar and keeps conversion O(log P).
- **Alternatives**: 10,000 (the spec's floor), which would refuse two coprime rules of a few hundred years each; 1,000,000, whose tables (~12 MB) are too big to hold for every calendar in a project.

## R4. The cycle table (the supercycle strategy)

- **Decision**: when a calendar is checked, the engine builds one table over the `P` instances of the era's unit whose counts from the epoch are `0 … P − 1` (years 1 … P):
  - each year's **variant**: which rules hold (evaluated once per rule per year). Years with the same set of holding rules share a variant, holding the members' counts after the changes, their prefix starts in fundamental units, and the year's length;
  - `starts[r]`: the fundamental units from the start of the cycle to the start of year `r`, with `starts[P]` the **cycle length**.
  
  For any year count `c`, its residue is `r = c mod⁺ P` (non-negative) and its cycle `k = ⌊c / P⌋`; the year starts at `k × cycle length + starts[r]` and has `r`'s variant. Instant to date finds `k` by floored division and `r` by binary search in `starts`. Both directions are O(log P), independent of the distance from the epoch (FR-008).
- **Why the residue gives the right rules**: a rule's period `P_i` divides `P`, and spec 005 guarantees the condition's value (or division-by-zero error) is the same at `y` and `y + P_i`. Year count `c` has number `c + 1`, and `c ≡ r (mod P)`, so its value is the one evaluated for year `r + 1`. Years far from the epoch are never evaluated directly, so a condition's arithmetic can't overflow there (spec Edge Cases, "Years far from the epoch").
- **Calendars without rules** use the same path with `P = 1` and one variant (the members as listed), so there is one conversion path, and spec 004's tests keep checking it.
- **Rationale**: this is the supercycle strategy of `/calendar/conversion.md`, with the period detected from the rules (spec 005) rather than stated.
- **Alternatives**: counting leap years in closed form per rule (only works for simple rules, and several rules interact); a table per year from the epoch (not O(1) in distance, and unbounded).

## R5. What is checked, and in what order

- **Decision**: rules are checked after spec 004's checks (so the shape is known), in this order, reporting the first fault and naming the rule by its number from 1:
  1. rules on any grouping other than the era's unit → `RulesNotOnEraUnit { unit }`;
  2. for each rule in order: its condition (`RuleCondition { unit, rule, error: CheckError }`), its member (`RuleUnknownMember { unit, rule, member }`), its change (`RuleZeroChange { unit, rule }`), its period (`RulePeriod { unit, rule, error: PeriodError }`);
  3. the combined period (`RulesPeriodOverLimit { unit, limit }`);
  4. the table, year by year for years 1 … P: a condition that fails (`RuleFails { unit, rule, year, error: EvalError }`), then a member length below 1 (`MemberTooShort { unit, member, year, length }`), then any total that doesn't fit `i64` (`BeyondLimits { unit, limit: Limit::CycleTooLong }`).
- **Which year is named**: the first year from 1 to P where it happens. Because the rules repeat every `P`, a failure anywhere in the range happens in that year too, and spec Story 1 scenarios 4 and 5 ask only for "a year where it happens".
- **Out of range while checking** (`year * 4611686018427387904 % 3 == 0` at year 2) is a failure like division by zero (`RuleFails` with `EvalError::OutOfRange`): FR-006 forbids meeting it later.
- **Rationale**: FR-005 and FR-006: everything that could go wrong in conversion is found here, once.

## R6. Conversion changes

- **Decision**:
  - **Instant to date**: the fixed number gives the era unit's count through the table (R4); units above the era's unit are then numbered from that count, dividing by their length **in instances of the era's unit** (10 years in a decade), not in fundamental units, because their length in fundamental units now varies (spec Edge Cases, "Units above the year"). Below the era's unit, the member unit and anything inside it use the year's variant instead of the listed counts.
  - **Date to instant**: the year's variant gives each member's valid range (FR-011: refused with the unit, value, range, member and the year it was checked against; `CalendarError::InvalidValue` gains a `year: Option<EraNumber>`, the era's unit's name and the number of the instance the value was checked against (so the message can say "in cycle 3" for a calendar whose era counts cycles), set whenever the calendar has rules and the unit is below the era's unit, `None` otherwise, so calendars without rules report exactly as before) and the member starts.
  - **`units_per`**: a count that depends on the variant (sols in a year, sols in a decade) is `NotFixed`; counts the rules don't change (months in a year, years in a decade) are still reported.
- **Rationale**: FR-009 to FR-011; spec 004's FR-008a for units above the era's unit.
- **Alternatives**: keeping above-era units in fundamental units; wrong once years vary.

## R7. Reporting a year's lengths (FR-010)

- **Decision**: `Calendar::era_instance(number: i64) -> Result<EraInstance, CalendarError>`, where `EraInstance { members: Vec<i64>, length: i64, holding: Vec<usize> }`: each member's count after the rules (in the unit the members count), the instance's length in fundamental units, and the numbers (from 1) of the rules that hold. For an era's unit that isn't a grouping it is `CalendarError::NotAGrouping { unit }`.
- **Rationale**: the calendar grid and converter can show leap years without evaluating rules themselves (FR-010), and tests can check Story 2 scenario 8 directly.

## R8. Tests (constitution VI)

- **Hand-worked**: every Story 1 and Story 2 scenario on Aster with its leap rule (spec), and a second calendar of different shape, **Tessel with a leap rule**: its third phase (14 turns) is shortened by 1 when `cycle % 3 == 0` (a shortening rule on an unnamed member, member "3", with a non-`year` era unit name), worked by hand around cycles 0, 3 and −3. Spec 004's tests run unchanged against calendars without rules.
- **Property tests** (`proptest`, 10,000 cases each; SC-002): random valid calendars from spec 004's generator, plus zero to three random repeating rules on the era's unit (conditions built as `(a * year + b) % m == c` joined by `&&`/`||`, changes of −2 … 3 kept valid by construction, periods within the limit). Properties: instant → date → instant and date → instant → date round trips, and agreement with a **year-by-year reference** in the test code that evaluates each rule directly for every year from the epoch (no cycle table), within ±2 rule periods plus 50 years of the epoch.
- **Independent oracle** (Story 3 scenario 4, SC-005): a Gregorian-shaped test calendar (86,400 s day, 365-day year of 12 months, February lengthened by 1 by the Gregorian rule, epoch at 1 January of year 1). A committed Python script, `crates/uni-calendar/tests/data/gregorian_days.py`, writes `gregorian_days.txt`: for each year 1 … 4000, Python's `datetime.date(y, 1, 1).toordinal()` and the 12 month lengths from `calendar.monthrange`, with the Python version in the file's header. The test converts every day of those years both ways and checks its fixed number is the ordinal − 1. Python's implementation shares no code with the engine, and its range (years 1 … 9999) covers ten Gregorian cycles.
- **Timing** (SC-004): the spec 004 timing check repeated on Gregorian-shaped and Aster-with-rule calendars, near the epoch and 100,000 years away, under 10 µs per conversion, run by hand in release mode.

## R9. Spec 004's range claim, and changes to knowledge

- **Range correction** (spec Assumptions): this design PR corrects spec 004's `research.md` R5 ("±100,000 years even with a 1 ns fundamental unit"), Story 4 scenario 1 and SC-002 ("±100,000 years" for any random calendar). `i64` fixed numbers cover about ±292 Earth years at 1 ns; ±100,000 Earth years needs a fundamental unit of at least about 343 ns, and ±100,000 Aster years (about 1.9 Earth years each) at least about 643 ns. The spec's own Edge Cases first gave 343 ns for Aster years; this PR corrects that too. Spec 004's property tests already use fundamental units of at least 1,000 ns, so no code changes. The corrected lines say the range depends on the fundamental unit's length and give both figures.
- **Knowledge**: `/calendar/leap-rules.md` gains a section "First implementation (spec 006)": rules attach to the era's unit for now, the predicate reads the unit's own number under its name, changes lengthen or shorten a member and add up, conditions must repeat, the rule period and its limit of 100,000, and every failure found when the calendar is checked. `/calendar/conversion.md`'s supercycle row says the period is the lcm of the rules' periods worked out by the expression language, with one table entry per instance of the era's unit and an O(log P) search, and that units above the era's unit are numbered by count of its instances. Both go back to `status: draft` for the owner to re-verify; a `knowledge/log.md` entry records the change. No change to `/architecture/crate-layout.md`: `uni-calendar → uni-expr` is already allowed.
