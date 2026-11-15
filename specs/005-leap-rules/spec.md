# Feature Specification: Leap Rules

**Feature Branch**: `design/005-leap-rules`

**Created**: 2026-11-15

**Status**: Approved by the owner, 2026-11-15

**Input**: User description: "Leap rules (issue #43, Phase 1 spec 005). The calendar engine (uni-calendar) supports leap rules: a rule attached to a grouping (such as the year) whose predicate, written in a first, minimal subset of the expression language (uni-expr: whole numbers and booleans, + - * floored % and division, comparisons, && || !, if/then/else, the unit's own number as a context variable), lengthens or shortens a named member by a whole number of children. Calendars with leap rules (Julian- and Gregorian-shaped) convert exactly in both directions, before and after the epoch, in constant time using the supercycle strategy, and dates that don't exist in a given year (a leap day in a common year) are refused with the reason. Out of scope: length and start expressions, inserting or removing members, intercalary members, user-defined functions, rational and other expression types, sky functions, the rule builder UI, parallel cycles, multiple eras, formats, presets, editing UI. Also carry spec 004's correction: research R5 and US4-1 wrongly claim ±100,000 years with a 1 ns fundamental unit; i64 fixed numbers cover about ±292 years at 1 ns (±100,000 years needs a fundamental unit of at least ~343 ns)."

**Issue**: #43 · **Phase**: 1. Calendar core · **Knowledge**: [/calendar/leap-rules.md](../../knowledge/calendar/leap-rules.md), [/language/expression-language.md](../../knowledge/language/expression-language.md), [/calendar/unit-kinds.md](../../knowledge/calendar/unit-kinds.md), [/calendar/conversion.md](../../knowledge/calendar/conversion.md), [/calendar/eras-and-epoch.md](../../knowledge/calendar/eras-and-epoch.md), [/calendar/gregorian-example.md](../../knowledge/calendar/gregorian-example.md), [/architecture/crate-layout.md](../../knowledge/architecture/crate-layout.md), [ADR 0008](../../knowledge/decisions/0008-physical-time-and-calendar-time.md)

## Clarifications

### Session 2026-11-15

- Q: What should the engine do with a leap rule that doesn't repeat on a fixed cycle (one using `year / 7` or `year * year`, unlike `year % 4 == 0`)? → A: Refuse it for now. The engine works out the cycle from the rule itself and refuses rules it can't show repeat, saying why (FR-011). Every real arithmetic leap rule repeats; non-repeating calendars (Hebrew-style postponements, observational ones) come later with start expressions or cached tables. The definer never states the cycle, so a wrong one can't give wrong dates silently.
- Q: Which units should be able to carry leap rules in this feature? → A: The era's unit only (the year), with rules reading only its number (FR-002, FR-006). That covers the Julian, Gregorian, Islamic-tabular and Persian arithmetic shapes. Rules on other groupings come later.
- Q: When several leap rules hold in the same year, what should happen? → A: All apply and their changes add up, whatever order the rules are listed in, so reordering rules never changes a calendar (FR-008). Combined lengths must stay at least 1 in every year, checked when the calendar is defined (FR-009).
- Q: Should this first expression-language subset include `let` bindings? → A: No, leave it for later. This feature's language is values, operators and `if … then … else` (FR-001); `let` and user-defined functions arrive with length and start expressions, where long formulas need them.

## User Scenarios & Testing *(mandatory)*

Like spec 004, this feature has no screen. Its users are the **features built on it**: the calendar presets (Gregorian, Julian, Islamic tabular), the converter, the calendar grid and the rule builder later in Phase 1, and the rule suggester of Phase 2. The **owner** sees its effect indirectly: every calendar whose year isn't a whole number of days needs leap rules, so this is the first feature that can describe calendars shaped like real ones.

Examples extend spec 004's made-up **Aster** calendar (sol of 88,775.244 s; 20 hours of 100 minutes; a year of 12 months, Prime … Twelfth, `[56, 56, 55, 56, 56, 55, 56, 56, 55, 56, 56, 55]` = 668 sols; decades and ages; one era "After Landing" numbering years) with one leap rule, shaped like the Gregorian one:

- **Aster leap rule**: when `year % 5 == 0 && (year % 100 != 0 || year % 500 == 0)`, lengthen **Twelfth** by 1 sol (55 → 56), so the year has 669 sols.
- So years 5, 10, 15 … are leap years, except 100, 200, 300 and 400; 500 is a leap year again. The pattern repeats every 500 years, which hold 96 leap years: 500 × 668 + 96 = 334,096 sols.
- Year numbers are astronomical (spec 004): year 0 is a leap year (0 is a multiple of 500), and so is year −5.

These numbers follow the ground rules in [ADR 0008](../../knowledge/decisions/0008-physical-time-and-calendar-time.md): the leap rule is the definer's choice, sits entirely on the calendar side, and is never checked against the planet's real orbit.

### User Story 1 - Write a rule as an expression (Priority: P1)

A leap rule's condition is written as text in the expression language, such as `year % 4 == 0`. The engine reads it, checks it, and works out its value for any year, refusing anything that is not a valid true/false condition and pointing at the exact place that is wrong.

**Why this priority**: every leap rule needs a condition, and the same expressions later drive cycles' skip rules, zone transitions and format computations. Without it there are no rules.

**Independent Test**: check and evaluate a set of conditions for chosen year numbers, including the Aster rule and deliberately broken ones, and compare with values worked out by hand.

**Acceptance Scenarios**:

1. **Given** the Aster rule's condition, **When** it's evaluated for years 5, 100, 500, 0, −5 and 1, **Then** the results are true, false, true, true, true and false.
2. **Given** `year % 4` (a whole number, not true/false), **When** it's checked as a rule condition, **Then** it's refused, saying a condition must be true or false.
3. **Given** `year % 4 == 0 &&` (incomplete), `month == 2` (no such variable for this rule) or `year + true` (mixed types), **When** each is checked, **Then** it's refused with the reason and the position (line and column) of the fault.
4. **Given** `-7 % 4` and `-7 / 4`, **When** they're evaluated, **Then** the results are 1 and −2 (floored, so rules behave the same before the epoch as after it).
5. **Given** `if year % 2 == 0 then year / 2 % 3 == 0 else false`, **When** it's evaluated for years 6, 4 and 3, **Then** the results are true, false and false.

---

### User Story 2 - Define a calendar with leap rules (Priority: P1)

Later features need to attach leap rules to a calendar's year: each rule pairs a condition with a change to one member (lengthen or shorten it by a whole number of the unit below). The engine checks the rules with the rest of the definition and refuses ones that can't work, saying why.

**Why this priority**: this is how a calendar gets leap years. With Story 1 it's the definition half of the feature.

**Independent Test**: build Aster with its leap rule and a few deliberately broken variants, and check that Aster is accepted and each broken one is refused with a reason that names the rule and what is wrong.

**Acceptance Scenarios**:

1. **Given** Aster with its leap rule, **When** it's checked, **Then** it's accepted; the engine reports that the year repeats every 500 years, in 334,096 sols, and that the number of sols in a year (or a decade) varies.
2. **Given** a rule that changes a member the year doesn't have (say "Thirteenth"), **When** it's checked, **Then** it's refused, naming the rule and the unknown member.
3. **Given** a rule that would shorten a member to zero sols or fewer in some year (say "shorten Third by 55 when `year % 2 == 0`"), **When** it's checked, **Then** it's refused, naming the rule, the member and a year where it happens.
4. **Given** a rule whose condition divides by zero, or overflows, for some year (say `100 / (year % 7) == 3`), **When** it's checked, **Then** it's refused, naming the rule, the reason and a year where it happens; conversion never meets such an error later.
5. **Given** a rule attached to a unit that has no members to change (the fundamental unit, or a subdivision), **When** it's checked, **Then** it's refused, saying why.
6. **Given** a rule whose condition doesn't repeat in a way the engine can work out, or repeats only over more years than the engine's limit (FR-011), **When** it's checked, **Then** it's refused, saying so.
7. **Given** two rules on the year, both of which hold in some year, **When** that year is converted, **Then** both changes apply (FR-008).

---

### User Story 3 - Convert dates in a calendar with leap rules (Priority: P1)

Instant-to-date and date-to-instant (spec 004) must take the rules into account: leap years are longer, the dates in them that exist only in leap years are valid there and refused elsewhere, and everything stays exact before and after the epoch.

**Why this priority**: rules are only useful if conversion obeys them. With Stories 1 and 2 it's the minimum useful feature.

**Independent Test**: convert instants and dates around leap years in Aster, at, after and before the epoch, and compare with dates worked out by hand.

**Acceptance Scenarios**:

1. **Given** year 5, Twelfth, sol 56, **When** it's converted to an instant and back, **Then** the same date comes back; it starts 3,340 sols after the epoch (4 × 668 + 668: the last sol of the 669-sol year 5).
2. **Given** year 1, Twelfth, sol 56, **When** it's converted, **Then** it's refused, saying Twelfth has only 55 sols in year 1.
3. **Given** the start of year 6, **When** it's converted to an instant, **Then** it's 3,341 sols after the epoch (5 × 668 + 1).
4. **Given** the start of year 501, **When** it's converted, **Then** it's 334,096 sols after the epoch, one full repetition of the rule.
5. **Given** the instant one nanosecond before the epoch, **When** it's converted, **Then** the date is year 0, Twelfth, sol 56 (year 0 is a leap year), hour 19, minute 99, with a remainder of one minute less one nanosecond, in decade 0 and age 0.
6. **Given** year 100, Twelfth, sol 56 (100 is not a leap year), **When** it's converted, **Then** it's refused; year 500, Twelfth, sol 56 is accepted.
7. **Given** an instant 100,000 years from the epoch, **When** it's converted in each direction, **Then** it takes no longer than one near the epoch (FR-012).

---

### User Story 4 - Exact conversion everywhere, with rules (Priority: P2)

The engine must stay exact over the whole supported range with leap rules too, and an independent check must confirm that a Gregorian-shaped calendar gives the same day numbers as an existing open-source implementation.

**Why this priority**: Stories 2 and 3 cover the behaviour; this story proves it holds everywhere, which constitution principles III and VI require.

**Independent Test**: run property tests over random calendars with random periodic leap rules, and compare a Gregorian-shaped test calendar with an independent implementation.

**Acceptance Scenarios**:

1. **Given** random valid calendars with random leap rules and random instants within the supported range (FR-015), **When** each instant is converted to a date and back, **Then** the original instant comes back exactly.
2. **Given** random valid dates in those calendars, **When** each is converted to an instant and back, **Then** the same date comes back.
3. **Given** a test calendar shaped like the proleptic Gregorian calendar (365-day year of 12 months, February lengthened by 1 day when `year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)`), **When** every day from year 1 to year 4000 (ten repetitions of the rule) is converted, **Then** its fixed number matches the day number from an independent open-source implementation, with the generator and its version committed. This is a test calendar, not the Gregorian preset, which comes later.

### Edge Cases

- **Before the epoch**: rules apply to year 0 and negative years exactly as to positive ones, using the same astronomical numbers the date shows; floored `%` and `/` keep patterns regular across zero (Story 1, scenario 4).
- **No rule holds**: a year in which no rule's condition holds has exactly the members' listed lengths, as in spec 004.
- **Several rules on one member**: their changes add up (FR-008). The combined length must stay at least 1 in every year (Story 2, scenario 3).
- **Shortening rules**: a rule may shorten a member as well as lengthen it (an "every 10th year loses a day" calendar), within the same limit.
- **Units above the year**: decades and ages are still numbered by their count of years from the epoch (spec 004, FR-008a); their length in sols now varies. Asking for a fixed count that now varies (sols in a year) reports that it varies, as spec 004 does for sols in a month.
- **Calendars without rules**: behave exactly as in spec 004, with the same results.
- **Range limits**: the supported range depends on the fundamental unit's length, because fixed numbers are whole numbers within a fixed range (about ±9.2 × 10¹⁸). With a 1 ns fundamental unit that is about ±292 Earth years; ±100,000 Aster-length years needs a fundamental unit of at least about 343 ns. Anything outside the range is an out-of-range error, never wrapped (spec 004, FR-015). This also corrects spec 004 (see Assumptions).

## Requirements *(mandatory)*

### Functional Requirements

**The expression language, first subset**

- **FR-001**: The expression language MUST support, for this feature: whole-number and true/false values; whole-number literals; `true` and `false`; `+`, `-` (binary and unary), `*`, floored `/` and floored `%`; comparisons `==`, `!=`, `<`, `<=`, `>`, `>=`; `&&`, `||`, `!` (with `&&` and `||` evaluating their right side only when needed); `if … then … else …`; and parentheses. This is a subset of `/language/expression-language.md`; other types, `let`, user-defined functions and sky functions come later.
- **FR-002**: An expression MUST be able to read **context variables** given by its user: for a leap rule, the number of the unit the rule is attached to, under that unit's own name (`year` in the Gregorian example, whatever the definer called it elsewhere).
- **FR-003**: Every expression MUST be checked before use: a syntax error, an unknown variable or a type error (such as `year + true`, or a condition that isn't true/false) MUST be refused with the reason and the exact position (line and column) of the fault.
- **FR-004**: Evaluation MUST be exact whole-number arithmetic, never floating point. Division or remainder by zero and results outside the supported whole-number range MUST be reported as errors, never wrapped.
- **FR-005**: The expression language MUST live in its own component (`uni-expr`), separate from the calendar, so that later features (cycles, zones, formats) can use it; it MUST depend only on what `/architecture/crate-layout.md` allows, and use no floating point.

**Leap rules**

- **FR-006**: A calendar's era's unit (the year), when it is a grouping with members, MUST be able to carry zero or more **leap rules**. Each rule has a **condition** (an expression that must be true/false, FR-003) and a **change**: lengthen or shorten one member, named or, if unnamed, given by its position, by a non-zero whole number of the unit directly below the grouping.
- **FR-007**: In each instance of the unit, the condition MUST be evaluated with that instance's number (FR-002); the change applies to that instance only when the condition is true.
- **FR-008**: When several rules apply to one instance, all their changes MUST apply, adding up, whatever order they're listed in.
- **FR-009**: The engine MUST check the rules with the rest of the definition (spec 004, FR-006) and refuse, naming the rule and the reason: a condition that fails FR-003; a change to a member the unit doesn't have; a change of zero; a rule on a unit that can't carry rules (FR-006); a combination of rules that would make any member's length zero or less in any instance; and a condition that would fail to evaluate (FR-004) for any instance. For these last two, the error MUST name an instance where it happens.
- **FR-010**: A checked calendar MUST never meet an expression error during conversion: every failure FR-009 can find is found when the calendar is checked.

**Repetition and conversion**

- **FR-011**: The engine MUST work out, from the rules themselves, the number of instances after which they repeat (the **rule period**: 500 years for Aster, 400 for the Gregorian shape). The definer never states the period. A condition whose repetition the engine can't establish (for example one that uses `year / 7` or `year * year` rather than remainders of `year`), or whose period exceeds the engine's limit (stated in planning, and at least 10,000 instances, enough for the longest cycles real arithmetic calendars use), MUST be refused with the reason.
- **FR-012**: Conversion MUST take time independent of how far the date is from the epoch, using the supercycle strategy of `/calendar/conversion.md`: one rule period of the era's unit is the repeating cycle.
- **FR-013**: Instant-to-date and date-to-instant (spec 004, FR-010 to FR-014) MUST take the rules into account: an instance's member lengths are the listed counts plus the changes of the rules that hold for it.
- **FR-013a**: The engine MUST report, for any instance of the era's unit, the length of each of its members and its total length after the rules, and which rules hold for it, so later features (the calendar grid, the converter) can show leap years without evaluating rules themselves.
- **FR-014**: A date with a value outside its unit's range in that instance (such as sol 56 of Twelfth in a common year) MUST be refused with an error naming the unit, the value, the valid range, the member and the instance of the era's unit it was checked against (spec 004, FR-013 and FR-019).
- **FR-015**: Every operation whose result would leave the supported range MUST report an out-of-range error (spec 004, FR-015). The supported range is limited by fixed numbers being whole numbers of fixed size, so it depends on the fundamental unit's length (Edge Cases).

**Quality**

- **FR-016**: The engine MUST be written test-first (principle VI), with tests against hand-worked dates for Aster with its leap rule and at least one other calendar of different shape, round-trip property tests in both directions over random calendars with random leap rules, and the Gregorian-shaped comparison with an independent open-source implementation (Story 4, scenario 3).
- **FR-017**: Errors MUST be structured (which rule, which unit, which member, which value, which range, which position, which reason) so later features can show a clear message.

### Key Entities

- **Expression**: text in the expression language, checked into a typed form before use, with errors pointing at a position.
- **Context variable**: a named value an expression can read; for a leap rule, the number of the unit it's attached to.
- **Leap rule**: a condition (true/false expression) and a change, attached to the era's unit.
- **Change**: lengthen or shorten one member by a non-zero whole number of the unit below the grouping.
- **Rule period**: the number of instances of the unit after which the rules repeat; one rule period is the supercycle used by conversion.
- **Instance lengths**: for one instance of the unit (one year), the length of each member after the rules that hold for it.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of hand-worked test dates and expression values convert or evaluate correctly, at, after and before the epoch.
- **SC-002**: Round-trip property tests run at least 10,000 random cases per property with no failures, across the whole supported range of each random calendar.
- **SC-003**: 100% of the invalid expressions, rules, definitions and dates in the tests are refused with an error naming the fault (and, for expressions, its position); none produces a value, date or instant.
- **SC-004**: A single conversion in a calendar with leap rules takes under 10 microseconds on the owner's machine, whether the date is near the epoch or 100,000 years away.
- **SC-005**: Every day from year 1 to year 4000 of the Gregorian-shaped test calendar has the same day number as the independent implementation.
- **SC-006**: Neither the expression language nor the calendar engine contains floating-point code, as confirmed by the architecture check.

## Assumptions

- **Engine only**: no screen, no storage, no editing UI, as in spec 004. The rule builder (`/language/rule-builder.md`) is a later feature; it will produce the same expressions.
- **Rules on the era's unit only**: leap rules attach to the era's unit (the year), whose number is what real leap rules test. Rules on other groupings, and rules that read more than the unit's own number, come later.
- **Changes are lengthen or shorten only**: inserting or removing members, intercalary members, length expressions and start expressions (`/calendar/leap-rules.md`) come later. Rules that don't repeat (the Hebrew calendar's postponements, observational rules) need start expressions or cumulative tables and are refused for now (FR-011).
- **No presets**: the Gregorian-shaped calendar in Story 4 is a test calendar. The Gregorian, Julian and Islamic-tabular presets are a later feature; this one gives them what they need.
- **Physical time and calendar time**: everything here is on the calendar side of ADR 0008. A leap rule is a convention of the calendar's definer, never derived from or checked against physics; matching it to an orbit is the rule suggester's job (Phase 2).
- **Correction carried from spec 004**: spec 004's `research.md` (R5) says whole-number fixed numbers cover ±100,000 years even with a 1 ns fundamental unit, and its Story 4, scenario 1 and SC-002 ask for ±100,000 years for any random calendar. Both are wrong: at 1 ns the range is about ±292 Earth years, and ±100,000 Aster-length years needs a fundamental unit of at least about 343 ns. The engine is correct (it reports out of range, spec 004 FR-015) and its property tests already use fundamental units of at least 1,000 ns. This feature's design PR corrects those lines in spec 004 and states the range as in Edge Cases above.
- **Knowledge update**: the design PR updates `/calendar/leap-rules.md` (rules attach to the era's unit for now, changes add up, the rule period), `/language/expression-language.md` (the first subset, floored division) and `/calendar/conversion.md` (one rule period as the supercycle), for the owner to re-verify.
