# Data Model: Expression Language Basics

All types live in `uni-expr`. No floating point anywhere. Nothing here is stored; storing expressions (source text and AST JSON) comes with a later feature.

## Inputs

### VarDecl

| Field | Type | Rule |
|---|---|---|
| `name` | `String` | A valid name (a letter or `_`, then letters, digits or `_`), not a keyword, unique among the declarations |
| `ty` | `Type` | `Int` or `Bool` |

### Value

`Int(i64)` or `Bool(bool)`. Passed to `eval`, one per declared variable, in declaration order.

### Type

`Int` (whole number) or `Bool` (true/false). Shown in messages as "whole number" and "true/false".

## Checked expression (`Expr`)

| Part | Meaning |
|---|---|
| source | The text exactly as given (FR-004) |
| tree | Typed nodes, each with its position; variables resolved to declaration indices |
| type | The result type, equal to the one the caller asked for |
| variables | The declarations it was checked against (for `period` and for checking `eval`'s inputs) |

Created only by `Expr::check`. Immutable; `Clone, Debug, PartialEq, Eq`.

### Nodes (internal)

| Node | Children | Type |
|---|---|---|
| Int literal | | whole number |
| Bool literal | | true/false |
| Variable (index) | | its declared type |
| Neg, Not | 1 | whole number, true/false |
| Add, Sub, Mul, Div, Rem | 2 | whole number |
| Lt, Le, Gt, Ge | 2 whole numbers | true/false |
| Eq, Ne | 2 of the same type | true/false |
| And, Or | 2 true/false | true/false |
| If | condition, then, else | the branches' type |

Each node keeps the position of its first character and, for operators, the operator's position (errors in evaluation point at the operator).

## Position (`Pos`)

| Field | Type | Meaning |
|---|---|---|
| `line` | `u32` | From 1 |
| `column` | `u32` | From 1, counted in characters, not bytes |

## Errors

### CheckError (from `Expr::check`; FR-003, FR-013)

| Variant | When | Example message |
|---|---|---|
| `InvalidVariable { name, reason }` | A declaration's name isn't valid, is a keyword, or is declared twice | "`if` can't be a variable name: it's a keyword" |
| `InvalidCharacter { found, at }` | A character that starts no token | "unexpected character `#` at 1:6" |
| `Unexpected { found, expected, at }` | A token where something else was needed; `found` is `None` at the end of the text | "expected a value but the expression ended at 1:17" |
| `LiteralOutOfRange { at }` | A literal outside `i64` | "number at 1:1 is outside the supported range" |
| `ChainedComparison { at }` | A second comparison operator in a row | "comparisons can't be chained at 1:10; use `&&`" |
| `TooDeep { limit, at }` | The tree passes the depth limit (256) | "expression is nested more than 256 levels deep at 3:40" |
| `UnknownVariable { name, at }` | A name that isn't declared | "unknown variable `month` at 1:1" |
| `TypeMismatch { context, expected, found, at }` | An operand, condition, branch or the whole result has the wrong type | "`+` needs a whole number, found true/false at 1:8" |

### EvalError (from `Expr::eval`; FR-006)

| Variant | When | Example message |
|---|---|---|
| `DivisionByZero { at }` | `/` or `%` by zero | "division by zero at 1:5" |
| `OutOfRange { at }` | A result outside `i64` | "result of `*` at 1:6 is outside the supported range" |
| `Inputs { expected, given }` | The values don't match the declarations in number or type (a caller bug, never a panic) | "expected 1 value(s) matching the declarations, got 2" |

### PeriodError (from `Expr::period`; FR-009)

| Variant | When | Example message |
|---|---|---|
| `NotRepeating { at }` | The form analysis meets a part that can't be shown to repeat (research R6) | "`/` at 1:6 stops the expression repeating as `year` counts up" |
| `OverLimit { limit }` | The period would be more than the caller's limit | "the expression repeats only after more than 1,000 steps" |
| `NotAWholeNumber { name }` | The variable isn't declared, or isn't a whole number | "`year` isn't a whole-number variable of this expression" |
| `BadLimit { limit }` | The limit is less than 1 | "the period limit must be at least 1" |

All three error types derive `Clone, Debug, PartialEq, Eq` and implement `Display` and `std::error::Error` by hand, like `uni-time` and `uni-calendar`.

## Relationships

- An `Expr` is checked against an ordered list of `VarDecl`s and an expected `Type`.
- `eval` takes one `Value` per `VarDecl`, in the same order, and gives a `Value` of the expression's type.
- `period` takes the name of one whole-number `VarDecl` and a limit, and gives a whole number from 1 to the limit.
