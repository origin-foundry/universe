---
type: Example
title: Gregorian example
description: The Gregorian calendar expressed in the calendar model.
tags: [calendar, example, gregorian, earth]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:22Z }
---

# Examples

This shows the model, not the storage format. Calendars live in SQLite; see the [storage schema](/storage/schema.md).

```yaml
calendar: Gregorian
clock: observer "Earth / Greenwich"   # τ on Earth's surface ≈ TT
epoch: start of first era (AD 1, January 1)
zones: Earth uses "zone rules" (IANA-style)
fundamental:
  unit: day
  kind: fixed
  duration: 86400 s
units:
  - year:
      kind: grouping
      children: month
      members: [January: 31, February: 28, March: 31, April: 30, May: 31, June: 30,
                July: 31, August: 31, September: 30, October: 31, November: 30, December: 31]
      rules:
        - when: "year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)"
          do: extend February by 1
  - day:
      kind: subdivision
      children: [hour × 24, minute × 60, second × 60]
cycles:
  - week: { period: 7, of: day, members: [Monday … Sunday], anchor: "fixed 1 = Monday" }
eras:
  - AD: { from: year 1, direction: up }          # first era → defines the epoch
pre_epoch:
  - BC: { direction: down, year_zero: false }
```

Concepts used: [calendar model](/calendar/calendar-model.md), [unit kinds](/calendar/unit-kinds.md), [leap rules](/calendar/leap-rules.md), [parallel cycles](/calendar/parallel-cycles.md), [eras and epoch](/calendar/eras-and-epoch.md), [time zones](/calendar/time-zones.md).
