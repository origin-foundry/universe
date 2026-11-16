---
type: Design
title: Leap rules
description: Predicate-and-modification rules, plus length and start expressions for complex calendars.
tags: [calendar, leap-rules, rules]
status: draft
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
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

# First implementation (spec 006)

The calendar engine supports the first, simplest kind of rule:

- **Where**: rules attach to the **era's unit** (the year, in most calendars) when it is a grouping. Rules on other units come later.
- **Predicate**: a true/false [expression](/language/expression-language.md) that reads one whole-number variable, the instance's number under the unit's own name (`year`, or whatever the definer called it), as the date shows it (astronomical numbering, so year 0 and negative years work the same way).
- **Modification**: lengthen or shorten one member, by name or by its number from 1, by a non-zero whole number of the unit its members count. When several rules hold in one instance, all their changes apply and add up, in any order.
- **Repetition**: every rule must repeat. The engine works out each predicate's period with the expression language and takes their least common multiple, the **rule period** (400 years for the Gregorian rule). The definer never states it. The rule period may be at most 100,000 instances; rules that don't repeat, or repeat only beyond that, are refused for now. Calendars whose rules don't repeat (Hebrew-style postponements, observational calendars) need the length and start expressions above, or cumulative tables.
- **Checked once**: when a calendar is checked, every predicate is evaluated for one rule period. A predicate that fails (division by zero, out of range), or changes that would make a member zero or less long, are refused, naming the rule and a year where it happens. A checked calendar never meets an expression error in conversion; every other year takes the value of the year a whole number of rule periods away.
