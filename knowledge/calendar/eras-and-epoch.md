---
type: Design
title: Eras and epoch
description: Multiple eras, the first era as epoch, and dates before the epoch.
tags: [calendar, eras, epoch]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:22Z }
---

# Eras

A calendar has an ordered list of **one or more eras**. Eras map the top-level unit's index to displayed numbers and labels:

- start (by fixed number, or by a timeline event, for regnal eras)
- numbering direction (counting up, or counting down to the era boundary)
- whether a year zero exists
- label and abbreviation per locale

# Epoch

**The first era defines the calendar's epoch.** The user places it on the hidden timeline by equating one of its dates with:

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
