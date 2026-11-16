---
type: Algorithm
title: Conversion
description: Converting between instants and dates, and the strategies that keep it fast.
tags: [calendar, conversion, performance]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-11-16T18:19:49Z }
---

# Instant to date

1. Map the instant to the calendar's clock (see [timeline and frames](/time/timeline-and-frames.md)).
2. Find the **fixed number**: for a fixed fundamental unit, by division; for an event-delimited one, by binary search in the cached event table.
3. Walk the unit tree top-down to resolve each field. Units above the era's unit are numbered by how many instances of the era's unit they hold (10 orbits in a cohort), because their length in fundamental units varies once the era's unit's does.

# Strategies for step 3

| Strategy | When it applies | Cost |
|---|---|---|
| **Supercycle** | The rules are periodic (e.g. Gregorian repeats every 400 years / 146,097 days). The engine works out the period from the rules themselves (the least common multiple of each predicate's period, found by the [expression language](/language/expression-language.md)) and precomputes one table over one period of the era's unit: each instance's member lengths after its rules, and where each instance starts within the period. A date's instance is found by one division and, for instant to date, a binary search within the period. With no adjustment rules the period is one instance of the era's unit. | O(1) in the distance from the epoch; O(log P) in the period |
| **Start expression** | The user supplied `start(unit)` in closed form; see [adjustment rules](/calendar/adjustment-rules.md). | O(log n) search over the index |
| **Cumulative table** | Everything else (observational, irregular). Lazily built, checkpointed, cached in the project file. | O(log n) lookup |

# Date to instant

The reverse uses the same machinery. Invalid dates (e.g. the 30th fundamental unit of a member only 29 long) are reported with the reason, never silently wrapped. A date may stop at any unit; smaller units then take their first value.

# Exactness and the one rounding

Unit boundaries are kept as exact fractions inside the engine; there is no floating point. Instants have whole nanoseconds, so a boundary can fall between two of them (a fundamental unit split into 7 subdivisions).

- **Date to instant**: when a date's exact start falls between two nanoseconds, the instant returned is the **first whole nanosecond at or after** that start, so it still lies inside the date. The result says it was rounded and gives the exact amount added (less than 1 ns). This is the only rounding in conversion.
- **Instant to date** never rounds: the date comes with the exact remainder of time since the start of its smallest unit, measured from the exact start.

So that the rounded instant always stays inside its date, every unit must last at least one nanosecond; the engine refuses definitions that don't, and states its other arithmetic limits the same way.

# Caching

Cumulative tables live in `calendar_table`; see the [storage schema](/storage/schema.md).
