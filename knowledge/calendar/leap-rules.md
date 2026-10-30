---
type: Design
title: Leap rules
description: Predicate-and-modification rules, plus length and start expressions for complex calendars.
tags: [calendar, leap-rules, rules]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:22Z }
---

# Rules

A rule is **predicate → modification**, attached to a unit:

- **Predicate:** an [expression](/language/expression-language.md) over the unit's index and context (e.g. `year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)`), or an event query for [observational calendars](/calendar/observational-calendars.md).
- **Modification:** extend or shorten a named member by *k* children; insert or remove a member at a position; insert an intercalary member.

Rules can be authored by hand (with the [rule builder](/language/rule-builder.md) or as expressions), or proposed by [rule suggestion](/calendar/rule-suggestion.md).

# Length and start expressions

For complex calendars (the Hebrew calendar is the reference stress test), a unit can instead define:

- a **length expression**: `length(year) = …`
- optionally a **start expression**: `start(year) = …`, giving the fixed number of the unit's first day in closed form

These use the expression language's pure functions, so molad and postponement logic can be written directly. A start expression also enables the fast path described in [conversion](/calendar/conversion.md).
