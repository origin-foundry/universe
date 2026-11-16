# Data Model: Leap Rules

All types live in `uni-calendar`, extending spec 004's [data model](../004-calendar-engine-basics/data-model.md); only what changes is listed here. Conditions are `uni-expr` expressions (spec 005). No floating point anywhere. Nothing here is stored.

## Definition (input)

### GroupingDef (changed)

| Field | Type | Rule |
|---|---|---|
| `rules` | `Vec<LeapRuleDef>` (new) | May be empty. Non-empty only on the grouping that is the era's unit (research R1) |

All other fields as in spec 004.

### LeapRuleDef (new)

| Field | Type | Rule |
|---|---|---|
| `condition` | `String` | A spec 005 expression with a true/false result, reading one whole-number variable named after the era's unit (research R2) |
| `member` | `String` | The member it changes: its name, or its number from 1 if it has none |
| `change` | `i64` | Not 0. Positive lengthens, negative shortens, in the unit the grouping's members count |

### Checks on rules (in order; research R5)

1. Only the era's unit has rules, and it is a grouping.
2. Each rule, in order: the condition checks (with the era's unit's name as its variable), the member exists, the change isn't 0, the condition repeats within the limit.
3. All the rules together repeat within the limit (100,000 instances).
4. For each year of one rule period (years 1 to P): every condition evaluates without error; every member's length after the changes is at least 1; every total fits in 64 bits.

## Checked calendar (internal)

| Part | Meaning |
|---|---|
| rules | The checked conditions (`uni_expr::Expr`), each with its member index and change |
| period `P` | The lcm of the rules' periods; 1 with no rules |
| variants | One per distinct set of holding rules: member counts after the changes, member starts in fundamental units, the instance's length, and which rules hold |
| residues | For each `r` in `0 … P − 1`: the variant of the instance whose count from the epoch is `r` (number `r + 1`) |
| cycle starts | For each `r` in `0 … P`: fundamental units from the start of a cycle to the start of instance `r`; the last is the cycle's length |
| above-era lengths | Each unit above the era's unit, as a count of the era's unit's instances (a decade is 10 years) |

Calendars without rules have `P = 1` and one variant: the members as listed.

## Output

### EraInstance (new)

What one instance of the era's unit looks like after its rules (FR-010).

| Field | Type | Meaning |
|---|---|---|
| `members` | `Vec<i64>` | Each member's count after the rules, in the unit the members count |
| `length` | `i64` | The instance's length in fundamental units |
| `holding` | `Vec<usize>` | The numbers (from 1) of the rules that hold, in order |

Aster, year 5: Twelfth is 56, `length` 669, `holding` `[1]`. Year 1: Twelfth is 55, `length` 668, `holding` `[]`.

### EraNumber (new)

An instance of the era's unit, named so an error message can say which one (FR-011, FR-015). The name and number travel together, so they can't disagree.

| Field | Type | Meaning |
|---|---|---|
| `unit` | `String` | The era's unit's name ("year", "cycle") |
| `number` | `i64` | The instance's astronomical number, as the date shows it |

## Errors

### DefinitionError (new variants; FR-005, FR-015)

Rules are numbered from 1 in the order listed. "Year" in messages is the era's unit's name.

| Variant | When | Example message |
|---|---|---|
| `RulesNotOnEraUnit { unit }` | A grouping other than the era's unit has rules, or the era's unit isn't a grouping | "decade can't have leap rules: rules go on the era's unit for now" |
| `RuleCondition { unit, rule, error: uni_expr::CheckError }` | The condition is refused by the expression language (including an era's unit name that isn't a valid variable name) | "rule 1 of year: unknown variable `month` at 1:1" |
| `RuleUnknownMember { unit, rule, member }` | The member isn't one of the unit's | "rule 1 of year changes Thirteenth, which year doesn't have" |
| `RuleZeroChange { unit, rule }` | The change is 0 | "rule 1 of year changes its member by 0" |
| `RulePeriod { unit, rule, error: uni_expr::PeriodError }` | The condition doesn't repeat, or repeats only beyond the limit | "rule 1 of year: the part at 1:6 stops the expression repeating as the variable counts up" |
| `RulesPeriodOverLimit { unit, limit }` | The rules together repeat only after more than the limit | "the rules of year repeat only after more than 100,000 years" |
| `RuleFails { unit, rule, year, error: uni_expr::EvalError }` | The condition fails to evaluate for some year | "rule 1 of year fails in year 7: division by zero at 1:5" |
| `MemberTooShort { unit, member, year, length }` | A member's length after the changes is below 1 in some year | "Third of year would be 0 long in year 2" |

`Limit` gains `CycleTooLong`: "one repetition of its rules is too long to count in fundamental units" (`BeyondLimits { unit, limit: CycleTooLong }`).

### CalendarError (changed; FR-011)

| Variant | Change |
|---|---|
| `InvalidValue { unit, value, first, last, within, year }` | New field `year: Option<EraNumber>`: the instance of the era's unit the value was checked against, set whenever the calendar has rules and the unit is below the era's unit; `None` otherwise, so calendars without rules report as in spec 004. Message adds " in " and the era unit's name and number when set: "sol 56 is outside 1 to 55 in Twelfth in year 1"; Tessel: "turn 13 is outside 0 to 12 in 3 in cycle 3" |
| `NotAGrouping { unit }` (new) | `era_instance` on a calendar whose era's unit isn't a grouping: "year isn't a grouping, so it has no members" |

## Relationships

- A `CalendarDef`'s era names a unit; if that unit is a grouping, its `GroupingDef.rules` are the calendar's leap rules.
- Each rule's member is one of that grouping's members; its change is in the unit those members count.
- Every instance of the era's unit has one variant: the residue of its count from the epoch modulo `P` picks it.
