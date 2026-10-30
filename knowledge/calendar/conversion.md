---
type: Algorithm
title: Conversion
description: Converting between instants and dates, and the strategies that keep it fast.
tags: [calendar, conversion, performance]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:22Z }
---

# Instant to date

1. Map the instant to the calendar's clock (see [timeline and frames](/time/timeline-and-frames.md)).
2. Find the **fixed number**: for a fixed fundamental unit, by division; for an event-delimited one, by binary search in the cached event table.
3. Walk the unit tree top-down to resolve each field.

# Strategies for step 3

| Strategy | When it applies | Cost |
|---|---|---|
| **Supercycle** | The rules are periodic (e.g. Gregorian repeats every 400 years / 146,097 days). The engine detects the period and precomputes one table. | O(1) |
| **Start expression** | The user supplied `start(unit)` in closed form; see [leap rules](/calendar/leap-rules.md). | O(log n) search over the index |
| **Cumulative table** | Everything else (observational, irregular). Lazily built, checkpointed, cached in the project file. | O(log n) lookup |

# Date to instant

The reverse uses the same machinery. Invalid dates (e.g. the 30th of a 29-day month) are reported with the reason, never silently wrapped.

# Caching

Cumulative tables live in `calendar_table`; see the [storage schema](/storage/schema.md).
