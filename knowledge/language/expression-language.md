---
type: Design
title: Expression language
description: The small, typed, side-effect-free language used for rules, cycles, zones and formats.
tags: [language, expressions, rules]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-11-16T18:19:49Z }
---

# Language

A small, typed, side-effect-free expression language, compiled to an AST and evaluated in Rust.

- **Types:** integer, rational, boolean, angle, duration, instant, string
- **Operators:** arithmetic, floored `/` and `%` (division rounds towards minus infinity; the remainder takes the sign of the divisor), comparisons, `&&`, `||`, `!`, `if … then … else`
- **Bindings:** `let x = … in …`
- **Pure user-defined functions**, so complex calendars can be built from reusable pieces. Recursion is allowed only with a statically bounded depth, so evaluation always terminates.
- **Context variables:** the current unit indices (`orbit`, `span`, …: the user's own unit names), era fields, the fixed number
- **Sky functions** (observational rules only): `elongation(a, b, from)`, `altitude(body, site)`, `phase(body)`, `distance(a, b)`, `event_between(kind, t0, t1)`

Every expression is type-checked when it's saved, with errors pointing at the exact position: a line and column, both from 1, with columns counted in characters.

# First subset

The first engine (spec 005) builds only part of the language:

- **Types:** integer (64-bit, checked: overflow and division by zero are errors, never wrapped) and boolean.
- **Operators**, loosest first: `if … then … else` (whose `else` extends as far as possible); `||`; `&&`; comparisons, which don't chain (`1 < x < 5` is refused); `+` and `-`; `*`, `/` and `%`; unary `-` and `!`. Same-level operators group left to right. `&&`, `||` and `if` evaluate only what they need.
- **Variables:** declared by the caller, each with a name and a type; for an adjustment rule, the number of the unit it tests. Names are a letter or `_` followed by letters, digits or `_`, in any script; `if`, `then`, `else`, `true` and `false` are keywords.
- **Depth:** an expression's tree is at most 256 levels deep, so evaluation can't exhaust the stack.

`let`, user-defined functions, the other types and sky functions come with the features that need them.

# Repetition

Fast conversion with adjustment rules (the supercycle in [conversion](/calendar/conversion.md)) needs to know when a rule repeats. The language works out a **period** *P* in one integer variable *y*: the expression's exact value, or division-by-zero error, is the same at *y* and *y + P* for every *y*. It is found from the expression itself, never stated by its author, by giving each part one of four forms:

| Form | Meaning |
|---|---|
| constant | doesn't depend on *y* |
| affine | exactly *a·y + b* |
| periodic (*P*) | repeats every *P* |
| not repeating | none of these can be shown |

An affine part becomes periodic only through `% m` with a constant *m* (period *m*, ignoring its sign); any operator over constant and periodic parts is periodic with the least common multiple of their periods. Dividing *y*, multiplying it by itself, comparing it or branching on it can't be shown to repeat, and is refused with the position of the part responsible. So the Gregorian rule `year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)` has period 400 and the tabular Islamic rule `(11 * year + 14) % 30 < 11` period 30, while `n / 7` and `n > 100` are refused. The period is always safe but not always the smallest; a caller gives a limit, and a longer period is refused. Out-of-range errors depend on how large the numbers are, not on the pattern, so callers evaluate one period near zero and map every other value onto it.

# Used by

- [Adjustment rules](/calendar/adjustment-rules.md), including length and start expressions
- [Parallel cycles](/calendar/parallel-cycles.md) skip rules
- [Observational calendars](/calendar/observational-calendars.md)
- [Time zones](/calendar/time-zones.md) recurring transitions
- Computed values in [format templates](/formats/templates.md)
- Custom events in the [event finder](/physics/event-finder.md)

# Storage

Expressions are stored as both source text and validated, versioned AST JSON. See the [storage schema](/storage/schema.md).
