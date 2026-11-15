# Feature Specification: Expression Language Basics

**Feature Branch**: `design/005-expression-language`

**Created**: 2026-11-15

**Status**: Approved by the owner, 2026-11-15

**Input**: Split out of leap rules (issue #43) during planning, by the owner's choice on 2026-11-15, so each build PR stays reviewable in about 30 minutes (constitution principle VIII). This spec takes the expression-language half of the approved leap-rules spec (its Story 1, FR-001 to FR-005, and the "work out the rule period" part of FR-011) and its clarifications about the language; leap rules themselves become spec 006, built on this one.

**Issue**: #44 · **Phase**: 1. Calendar core · **Knowledge**: [/language/expression-language.md](../../knowledge/language/expression-language.md), [/calendar/leap-rules.md](../../knowledge/calendar/leap-rules.md), [/calendar/conversion.md](../../knowledge/calendar/conversion.md), [/architecture/crate-layout.md](../../knowledge/architecture/crate-layout.md)

## Clarifications

### Session 2026-11-15

These were asked while specifying leap rules and approved with that spec; they carry over to this half.

- Q: What should happen with a leap rule that doesn't repeat on a fixed cycle (one using `year / 7` or `year * year`, unlike `year % 4 == 0`)? → A: Refuse it for now. The engine works out the cycle from the rule itself and refuses rules it can't show repeat, saying why; the definer never states the cycle, so a wrong one can't give wrong dates silently. For this spec: the language works out the repetition (FR-008) and says where it can't (FR-009).
- Q: Should this first expression-language subset include `let` bindings? → A: No, leave it for later. This subset is values, operators and `if … then … else` (FR-001); `let` and user-defined functions arrive with length and start expressions, where long formulas need them.
- Q: (Owner, in planning) Leap rules came to about twice the size of a 30-minute build PR; how should it go ahead? → A: Split in two: this spec (the expression language, including working out repetition) first, then spec 006 (leap rules), each with its own design and build PR.

## User Scenarios & Testing *(mandatory)*

This feature has no screen. Its users are the **features built on it**: leap rules (spec 006) first, then parallel cycles' skip rules, zone transitions, computed values in formats and the rule builder, all of which write conditions and formulas in this language (`/language/expression-language.md`). The **owner** sees its effect indirectly: every rule a calendar definer writes is checked, explained and evaluated here.

Examples use the leap rule from spec 006's made-up **Aster** calendar: `year % 5 == 0 && (year % 100 != 0 || year % 500 == 0)`, where `year` is a whole-number variable supplied by the caller (the number of the year being tested). Year numbers can be zero or negative (astronomical numbering, spec 004).

### User Story 1 - Check an expression (Priority: P1)

A feature that accepts an expression (such as a leap rule's condition) gives the language the text, the variables the expression may read with their types, and the type the result must have. The language accepts it or refuses it, saying what is wrong and exactly where.

**Why this priority**: an expression that can't be checked can't be trusted; principle VII requires faults to be reported with the reason, and the rule builder and editor will point users at the exact position.

**Independent Test**: check a set of valid and deliberately broken expressions against `year` (a whole number) and an expected true/false result, and compare each result or error (reason, line, column) with what's expected.

**Acceptance Scenarios**:

1. **Given** the Aster rule, with `year` as a whole-number variable and a true/false result expected, **When** it's checked, **Then** it's accepted.
2. **Given** `year % 4` with a true/false result expected, **When** it's checked, **Then** it's refused, saying a true/false value was expected and a whole number found, at line 1, column 1.
3. **Given** `year % 4 == 0 &&`, **When** it's checked, **Then** it's refused, saying the expression ended where a value was expected, at line 1, column 17.
4. **Given** `month == 2` when only `year` is available, **When** it's checked, **Then** it's refused, naming the unknown variable `month` and its position.
5. **Given** `year + true`, `!year` or `if year then 1 else 2`, **When** each is checked, **Then** it's refused, naming the operator or keyword, the type expected and the type found, and the position.
6. **Given** an expression over two lines, **When** it has a fault on the second line, **Then** the line and column given point at that fault.
7. **Given** `1 < year < 5`, **When** it's checked, **Then** it's refused, saying comparisons can't be chained (`(1 < year) && (year < 5)` is the way to write it).
8. **Given** a whole-number literal too large for the supported range (say 99999999999999999999), **When** it's checked, **Then** it's refused with its position, never wrapped.

---

### User Story 2 - Evaluate an expression (Priority: P1)

A feature that has a checked expression evaluates it with values for its variables and gets the exact result, or an error saying why there isn't one.

**Why this priority**: with Story 1, this is the minimum useful language; leap rules evaluate their conditions this way.

**Independent Test**: evaluate checked expressions for chosen variable values and compare with results worked out by hand.

**Acceptance Scenarios**:

1. **Given** the Aster rule, **When** it's evaluated for years 5, 100, 500, 0, −5 and 1, **Then** the results are true, false, true, true, true and false.
2. **Given** `-7 % 4`, `-7 / 4`, `7 % -4` and `7 / -4`, **When** they're evaluated, **Then** the results are 1, −2, −1 and −2: division rounds down and the remainder takes the sign of the divisor (floored), so patterns stay regular across year zero.
3. **Given** `if year % 2 == 0 then year / 2 % 3 == 0 else false`, **When** it's evaluated for years 6, 4 and 3, **Then** the results are true, false and false.
4. **Given** `100 / (year % 7) == 3`, **When** it's evaluated for year 14, **Then** it's refused as a division by zero, with the position of the `/`.
5. **Given** `year * 4611686018427387904`, **When** it's evaluated for year 2, **Then** it's refused as out of range, with the position of the `*`, never wrapped.
6. **Given** `year % 2 == 0 && 100 / (year % 7) == 3`, **When** it's evaluated for year 7, **Then** the result is false with no error: `&&` and `||` evaluate their right side only when the left side doesn't already decide the result.

---

### User Story 3 - Work out how an expression repeats (Priority: P1)

Leap rules (spec 006) convert dates fast only if their conditions repeat on a fixed cycle as the year counts up. Given a checked expression and one of its whole-number variables, the language works out a number of steps after which the expression's value always repeats (its **period** in that variable), or says why it can't.

**Why this priority**: spec 006 refuses rules that don't repeat (Clarifications), and it can only do that if this works; the definer never states the period.

**Independent Test**: work out the period of a set of expressions in `year`, compare with periods worked out by hand, and check that each refused one names the part that doesn't repeat.

**Acceptance Scenarios**:

1. **Given** the Aster rule, **When** its period in `year` is worked out, **Then** it's 500.
2. **Given** `year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)` (the Gregorian shape), **When** its period is worked out, **Then** it's 400.
3. **Given** `(11 * year + 14) % 30 < 11` (the shape of the tabular Islamic leap rule), **When** its period is worked out, **Then** it's 30.
4. **Given** `true` or `3 > 2`, which don't read `year` at all, **When** their period is worked out, **Then** it's 1.
5. **Given** `year / 7 % 2 == 0`, `year * year % 3 == 0` or `year > 100`, **When** its period is worked out, **Then** it's refused, saying it doesn't repeat in a way the language can establish, and pointing at the part responsible (the `/`, the `*`, the `>`).
6. **Given** an expression whose period would exceed a limit the caller gives (say `year % 7 == 0 && year % 11 == 0 && year % 13 == 0`, period 1,001, with a limit of 1,000), **When** its period is worked out, **Then** it's refused, saying the period is over the limit.
7. **Given** any expression the language says has period *P* and any year *y*, **When** it's evaluated at *y* and at *y + P*, **Then** both give the same result (or both the same error).

---

### User Story 4 - Exact and safe on any input (Priority: P2)

The language must never use floating point, never wrap a number, and never crash, whatever text it's given.

**Why this priority**: Stories 1 to 3 cover the behaviour; this story proves it holds everywhere, which constitution principles III, VI and VII require. Expressions will be typed by users, so any text at all must give a clear answer.

**Independent Test**: property tests over random expressions and random text.

**Acceptance Scenarios**:

1. **Given** random valid expressions built from the subset, **When** each is evaluated, **Then** the result matches an independent reference evaluator that uses unlimited-size numbers, or the language reports out of range exactly when the reference result doesn't fit.
2. **Given** random expressions whose period the language works out, **When** each is evaluated at random *y* and *y + P*, **Then** the results agree (Story 3, scenario 7).
3. **Given** random text, including very long and very deeply nested text, **When** it's checked, **Then** it's accepted or refused with a reason and position; nothing crashes.

### Edge Cases

- **Whitespace and line breaks** are allowed between any two tokens; positions count lines from 1 and columns from 1, in characters (not bytes), so names in any script point at the right place.
- **Names**: a variable name is a letter or `_` followed by letters, digits or `_`, in any script; `if`, `then`, `else`, `true` and `false` are keywords and can't be variable names. A caller offering a variable whose name isn't a valid name gets an error saying so.
- **Precedence**, loosest first: `if … then … else …`; `||`; `&&`; comparisons (which don't chain); `+` and `-`; `*`, `/` and `%`; unary `-` and `!`. Operators of the same level group left to right. Parentheses override.
- **Equality** compares two whole numbers or two true/false values; comparing a number with a true/false value is a type error.
- **`if` branches** must have the same type; the result has that type.
- **Range**: whole numbers are 64-bit signed, about ±9.2 × 10¹⁸. A literal or any result outside that range is an error (Story 1 scenario 8, Story 2 scenario 5). The smallest number divided by −1 is out of range too.
- **Depth**: nesting deeper than a stated limit (set in planning, at least 64 levels) is refused with its position, so deeply nested text can't exhaust the stack.
- **Repetition is a safe answer, not always the smallest**: the period worked out always works (Story 3 scenario 7) but may be a multiple of the smallest one; `year % 4 == 0 || year % 2 == 0` may report 4 even though its smallest period is 2. Callers that care can search for a smaller one inside the period given.
- **Repetition and errors**: an expression can repeat and still fail for some values (Story 2 scenario 4 has period 7 and fails for every multiple of 7). Working out the period doesn't evaluate; a caller that needs every value checks one period's worth.
- **Repetition and range**: the period describes the expression's exact meaning. Evaluating with very large variable values can overflow even when the expression repeats (`year * 1000000000000 % 7`); the value at *y* is then still the value at *y* minus a multiple of *P*, which a caller like spec 006 can use instead.

## Requirements *(mandatory)*

### Functional Requirements

**The language, first subset**

- **FR-001**: The language MUST support: whole-number and true/false values; whole-number literals; `true` and `false`; `+`, `-` (binary and unary), `*`, floored `/` and floored `%`; comparisons `==`, `!=`, `<`, `<=`, `>`, `>=`; `&&`, `||`, `!`; `if … then … else …`; parentheses; and variables. This is a subset of `/language/expression-language.md`; other types, `let`, user-defined functions and sky functions come later.
- **FR-002**: The caller MUST declare the **variables** an expression may read, each with a name and a type (whole number or true/false), and the type the result must have. Only declared variables may be used.

**Checking**

- **FR-003**: Every expression MUST be checked before use. Syntax errors, unknown variables, type errors (including a result of the wrong type), chained comparisons, out-of-range literals and nesting beyond the depth limit MUST be refused with the reason and the exact position (line and column) of the fault.
- **FR-004**: A checked expression MUST keep its source text, so later features can show and store it as written.

**Evaluation**

- **FR-005**: Evaluating a checked expression with a value for each declared variable MUST give its exact result. Arithmetic is exact 64-bit whole-number arithmetic, never floating point; `/` rounds towards minus infinity and `%` takes the sign of the divisor (floored).
- **FR-006**: Division or remainder by zero, and any result outside the supported range, MUST be an error naming the reason and the position of the operator, never a wrapped value.
- **FR-007**: `&&` and `||` MUST evaluate their right side only when the left side doesn't decide the result, and `if` MUST evaluate only the branch it takes, so errors in parts not evaluated are not raised.

**Repetition**

- **FR-008**: The language MUST work out, for a checked expression and one of its whole-number variables, a **period** *P* ≥ 1 such that for every value *y* the expression's exact value (or error) at *y* and at *y + P* is the same, with the other variables unchanged. It MUST establish this from the expression itself; it is never supplied by the caller.
- **FR-009**: When the language can't establish a period (because the variable is divided, multiplied by itself, compared, or otherwise used other than through a remainder by a fixed number), or the period would exceed a limit given by the caller, it MUST refuse with the reason and the position of the part responsible.

**Quality**

- **FR-010**: The language MUST live in its own component (`uni-expr`), depending only on what `/architecture/crate-layout.md` allows, and MUST NOT use floating point (it's an exact crate).
- **FR-011**: No input text, variable value or limit may make it crash; every failure is a structured error.
- **FR-012**: It MUST be written test-first (principle VI), with tests against hand-worked values for every scenario above, and property tests comparing evaluation with an independent reference evaluator and checking every period worked out.
- **FR-013**: Errors MUST be structured (which reason, which operator, which types, which name, which position) so later features can show a clear message.

### Key Entities

- **Expression**: source text, checked into a typed form before use.
- **Variable**: a name and a type the caller declares; given a value at evaluation.
- **Type**: whole number or true/false.
- **Value**: a whole number or true/false.
- **Position**: line and column, from 1, in characters.
- **Period**: a number of steps of one whole-number variable after which the expression's value always repeats.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of the hand-worked expressions in the tests check, evaluate and give their period as expected.
- **SC-002**: 100% of the invalid expressions in the tests are refused with the expected reason and position; none produces a value.
- **SC-003**: Property tests run at least 10,000 random cases per property with no failures: evaluation against the reference, periods, and random text.
- **SC-004**: Evaluating a leap-rule-sized expression (like the Gregorian rule) takes under 1 microsecond on the owner's machine, so checking a 10,000-year period takes under 10 milliseconds.
- **SC-005**: The language contains no floating-point code, as confirmed by the architecture check.

## Assumptions

- **Engine only**: no screen, no storage, no editing UI. Storing expressions as source text and versioned AST JSON (`/storage/schema.md`) comes with the first feature that stores calendars; this feature keeps the source text (FR-004) so that is possible.
- **Only what leap rules need**: the subset is what spec 006 needs plus `if … then … else` (already in the approved leap-rules spec). The other types and constructs in `/language/expression-language.md` arrive with the features that need them.
- **No dependency on the calendar**: the language knows nothing about calendars; "year" is just a variable name its caller declares. Spec 006 decides which variables a leap rule sees and what limit its period has.
- **Physical time and calendar time** (ADR 0008): the language sits on neither side; it computes with numbers it is given. Sky functions, which would bring physical time in, come in Phase 3.
- **Knowledge update**: the design PR updates `/language/expression-language.md` with the first subset, floored division, positions, the depth limit and how repetition is worked out, for the owner to re-verify. The pending correction to spec 004's range wording (research R5, Story 4 scenario 1, SC-002) stays with spec 006, the next design PR that touches the calendar engine, as the owner chose.
