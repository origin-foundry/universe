---
type: Design
title: Expression language
description: The small, typed, side-effect-free language used for rules, cycles, zones and formats.
tags: [language, expressions, rules]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:54Z }
---

# Language

A small, typed, side-effect-free expression language, compiled to an AST and evaluated in Rust.

- **Types:** integer, rational, boolean, angle, duration, instant, string
- **Operators:** arithmetic, `%` (floored modulo), comparisons, `&&`, `||`, `!`, `if … then … else`
- **Bindings:** `let x = … in …`
- **Pure user-defined functions**, so complex calendars can be built from reusable pieces. Recursion is allowed only with a statically bounded depth, so evaluation always terminates.
- **Context variables:** the current unit indices (`year`, `month`, … using the user's own unit names), era fields, the fixed number
- **Sky functions** (observational rules only): `elongation(a, b, from)`, `altitude(body, site)`, `phase(body)`, `distance(a, b)`, `event_between(kind, t0, t1)`

Every expression is type-checked when it's saved, with errors pointing at the exact position.

# Used by

- [Leap rules](/calendar/leap-rules.md), including length and start expressions
- [Parallel cycles](/calendar/parallel-cycles.md) skip rules
- [Observational calendars](/calendar/observational-calendars.md)
- [Time zones](/calendar/time-zones.md) recurring transitions
- Computed values in [format templates](/formats/templates.md)
- Custom events in the [event finder](/physics/event-finder.md)

# Storage

Expressions are stored as both source text and validated, versioned AST JSON. See the [storage schema](/storage/schema.md).
