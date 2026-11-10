---
type: Design
title: Unit kinds
description: Subdivisions, groupings, event-delimited groupings and intercalary members.
tags: [calendar, units]
status: draft
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
---

# Kinds

| Kind | Description | Earth example |
|---|---|---|
| **Subdivision** | Parent split into *n* equal children. Can apply to event-delimited parents, giving variable-length "temporal hours". | day → 24 hours → 60 minutes |
| **Grouping** | A sequence of **members**, each a count of the unit directly below the grouping, with optional names. When there is more than one member, the grouping names its **member unit**, the unit its members are instances of; the member unit has no definition of its own. Counts can be a fixed list, a list adjusted by [leap rules](/calendar/leap-rules.md), or a **length expression**. | year → 12 months [31, 28, 31, …] days (member unit: month); decade → 10 years (one member, no member unit) |
| **Event-delimited grouping** | Boundaries come from physical events, snapped to the fundamental unit. See [observational calendars](/calendar/observational-calendars.md). | Observational lunar month |
| **Intercalary member** | A member that belongs to its parent but sits outside the regular sequence (and can be excluded from [parallel cycles](/calendar/parallel-cycles.md)). | Epagomenal days; the World Calendar's "Worldsday" |

# Numbering within a parent

Each unit below the [era's unit](/calendar/eras-and-epoch.md) has a **first number**: the number of its first instance within its parent. It can be any whole number, positive, zero or negative. If not given, it is 1 for units at or above the fundamental unit (months, days) and 0 for subdivisions (hours, minutes). Valid values then run from the first number to first number + count − 1, and anything outside that range is refused, never wrapped. The first number is part of the date's meaning; how a number is shown (padding, names, ordinals) belongs to [formats](/formats/templates.md).

# Structure

Units form a **tree** under the top-level unit, rooted on the fundamental unit described in the [calendar model](/calendar/calendar-model.md). Parallel cycles hang off the tree without nesting.

The first engine (spec 004) builds the simplest tree, a single chain: fixed-count groupings above the fundamental unit (each with an optional member unit) and equal subdivisions below it, such as year → month → day → hour → minute. Branching trees, intercalary members and event-delimited groupings come later.
