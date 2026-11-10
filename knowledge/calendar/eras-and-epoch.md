---
type: Design
title: Eras and epoch
description: Multiple eras, the first era as epoch, and dates before the epoch.
tags: [calendar, eras, epoch]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-11-10T22:41:49Z }
---

# Eras

A calendar has an ordered list of **one or more eras**. The eras number a **chosen unit**, the **era's unit** (usually the year), which can be any unit at or above the fundamental unit. Eras map the era's unit's index to displayed numbers and labels:

- start (by fixed number, or by a timeline event, for regnal eras)
- numbering direction (counting up, or counting down to the era boundary)
- whether a year zero exists
- label and abbreviation per locale

# Units above the era's unit

Fixed-count groupings can sit above the era's unit (decade = 10 years, century = 10 decades). Each is numbered by its **count from the epoch**, with the same astronomical numbering: the one that starts at the epoch is 1, the one before it 0, then −1. So century 1 holds years 1–100 and century 0 holds years −99 to 0. These numbers are worked out from the date, never entered; a date starts at the era's unit.

# Epoch

**The first era defines the calendar's epoch**: the start of its first date (the first instance of the era's unit and of every unit below it) is fixed number 0. The user places it on the hidden timeline by equating one of its dates with:

- a date in another calendar,
- a timeline event, or
- a sky event (e.g. "year 1 began at the first equinox after the Landing").

Later eras start relative to the calendar's own dates. See [ADR 0002](/decisions/0002-hidden-absolute-timeline.md).

# Time before the epoch

Dates before the first era are always valid; the calendar's rules simply run backwards (**proleptic** extension). Labelling is configurable per calendar:

- a **pre-epoch era** (e.g. "Before Founding", counting down, with or without a year zero)
- **astronomical numbering** (year 0, −1, −2, …)
- both, with formats choosing which to show

Leap rules, parallel cycles and observational units all apply before the epoch exactly as after it (observational ones as far back as the ephemeris is computed).
