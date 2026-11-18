---
type: Design
title: Eras and epoch
description: Multiple eras, the first era as epoch, and dates before the epoch.
tags: [calendar, eras, epoch]
status: draft
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
---

# Eras

A calendar has an ordered list of **one or more eras**. The eras number a **chosen unit**, the **era's unit** (on Earth, the year), which can be any unit at or above the fundamental unit. Eras map the era's unit's index to displayed numbers and labels:

- start (by fixed number, or by a timeline event, for regnal eras)
- numbering direction (counting up, or counting down to the era boundary)
- whether an instance zero exists
- label and abbreviation per locale

# Units above the era's unit

Fixed-count groupings can sit above the era's unit (a cohort of 10 orbits, an age of 10 cohorts). Each is numbered by its **count from the epoch**, with the same astronomical numbering: the one that starts at the epoch is 1, the one before it 0, then −1. So age 1 holds orbits 1–100 and age 0 holds orbits −99 to 0. These numbers are worked out from the date, never entered; a date starts at the era's unit.

# Epoch

**The first era defines the calendar's epoch**: the start of its first date (the first instance of the era's unit and of every unit below it) is fixed number 0. The user places it on the hidden timeline by equating one of its dates with:

- a date in another calendar,
- a timeline event, or
- a sky event (e.g. "orbit 1 began at the first equinox after the Landing").

Later eras start relative to the calendar's own dates. See [ADR 0002](/decisions/0002-hidden-absolute-timeline.md).

## Anchoring by calendar dates (first implementation, spec 008)

- **The reference calendar**: the first calendar added to an empty universe has no anchor. Its epoch is the hidden zero. Other calendars are placed from it, directly or through each other.
- **Anchors**: every other calendar equates one of its dates with a date in another calendar of the universe, meaning the two dates start at the same moment. Both dates are entered and shown as dates, never as timeline values. An anchor is one of two kinds:
  - **Fix the moment** (the default): the moment is worked out when the anchor is confirmed and kept. Editing the other calendar later doesn't move this one; the editor shows where the moment now falls in the other calendar. Editing this calendar keeps its own anchoring date at the moment.
  - **Keep the equation**: the calendar follows the other calendar's edits, so its anchoring date always starts at the other calendar's date. Equations may chain but not loop.
- **Deleting a calendar** never moves another. Calendars that kept an equation with it are switched to fixed moments where they stand, and every anchor that named it keeps its dates, noting the calendar is gone.
- **Exactness**: the epoch is a whole nanosecond on the timeline, chosen so the two dates' starts land on the same nanosecond. Where a date's exact start falls between two nanoseconds, the engine's rounding rule and report apply (see [conversion](/calendar/conversion.md)).
- Anchoring to timeline and sky events comes with events and physics.

# Time before the epoch

Dates before the first era are always valid; the calendar's rules simply run backwards (**proleptic** extension). Labelling is configurable per calendar:

- a **pre-epoch era** (e.g. "Before Founding", counting down, with or without an instance zero)
- **astronomical numbering** (0, −1, −2, … with no gap; Earth's astronomical year numbering)
- both, with formats choosing which to show

Adjustment rules, parallel cycles and observational units all apply before the epoch exactly as after it (observational ones as far back as the ephemeris is computed).
