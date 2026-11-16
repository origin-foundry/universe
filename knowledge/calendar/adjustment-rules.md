---
type: Design
title: Adjustment rules
description: Predicate-and-modification rules that adjust a unit's members, plus length and start expressions for complex calendars.
tags: [calendar, adjustment-rules, rules]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-11-16T18:19:49Z }
---

An **adjustment rule** changes a unit's shape in the instances where it holds. On Earth these are leap rules, such as the Gregorian calendar's extra day in February; [ADR 0009](/decisions/0009-earth-neutral-names.md) names them generically, because they lengthen and shorten.

# Rules

A rule is **predicate → modification**, attached to a unit:

- **Predicate:** an [expression](/language/expression-language.md) over the unit's index and context (e.g. `orbit % 4 == 0 && (orbit % 100 != 0 || orbit % 400 == 0)` for a unit named orbit; the Gregorian rule is the same with `year`), or an event query for [observational calendars](/calendar/observational-calendars.md).
- **Modification:** extend or shorten a named member by *k* children; insert or remove a member at a position; insert an intercalary member.

Rules can be authored by hand (with the [rule builder](/language/rule-builder.md) or as expressions), or proposed by [rule suggestion](/calendar/rule-suggestion.md).

# Length and start expressions

For complex calendars (the Hebrew calendar is the reference stress test), a unit can instead define:

- a **length expression**: `length(unit) = …`
- optionally a **start expression**: `start(unit) = …`, giving the fixed number of the unit's first fundamental unit in closed form

These use the expression language's pure functions, so molad and postponement logic can be written directly. A start expression also enables the fast path described in [conversion](/calendar/conversion.md).

# First implementation (spec 006)

The calendar engine supports the first, simplest kind of rule:

- **Where**: rules attach to the **era's unit** (on Earth, the year) when it is a grouping. Rules on other units come later.
- **Predicate**: a true/false [expression](/language/expression-language.md) that reads one whole-number variable, the instance's number under the unit's own name (`orbit`, or whatever the definer called it), as the date shows it (astronomical numbering, so instance 0 and negative instances work the same way).
- **Modification**: lengthen or shorten one member, by name or by its number from 1, by a non-zero whole number of the unit its members count. When several rules hold in one instance, all their changes apply and add up, in any order.
- **Repetition**: every rule must repeat. The engine works out each predicate's period with the expression language and takes their least common multiple, the **rule period** (400 for the Gregorian rule). The definer never states it. The rule period may be at most 100,000 instances; rules that don't repeat, or repeat only beyond that, are refused for now. Calendars whose rules don't repeat (Hebrew-style postponements, observational calendars) need the length and start expressions above, or cumulative tables.
- **Checked once**: when a calendar is checked, every predicate is evaluated for one rule period. A predicate that fails (division by zero, out of range), or changes that would make a member zero or less long, are refused, naming the rule and an instance where it happens. A checked calendar never meets an expression error in conversion; every other instance takes the value of the instance a whole number of rule periods away.
