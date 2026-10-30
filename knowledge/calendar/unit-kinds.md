---
type: Design
title: Unit kinds
description: Subdivisions, groupings, event-delimited groupings and intercalary members.
tags: [calendar, units]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:22Z }
---

# Kinds

| Kind | Description | Earth example |
|---|---|---|
| **Subdivision** | Parent split into *n* equal children. Can apply to event-delimited parents, giving variable-length "temporal hours". | day → 24 hours → 60 minutes |
| **Grouping** | A sequence of child counts per position, with optional **named members**. Counts can be a fixed list, a list adjusted by [leap rules](/calendar/leap-rules.md), or a **length expression**. | year → 12 months [31, 28, 31, …] |
| **Event-delimited grouping** | Boundaries come from physical events, snapped to the fundamental unit. See [observational calendars](/calendar/observational-calendars.md). | Observational lunar month |
| **Intercalary member** | A member that belongs to its parent but sits outside the regular sequence (and can be excluded from [parallel cycles](/calendar/parallel-cycles.md)). | Epagomenal days; the World Calendar's "Worldsday" |

# Structure

Units form a **tree** under the top-level unit, rooted on the fundamental unit described in the [calendar model](/calendar/calendar-model.md). Parallel cycles hang off the tree without nesting.
