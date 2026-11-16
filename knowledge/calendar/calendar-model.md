---
type: Design
title: Calendar model
description: What a calendar is made of, its clock, and the fundamental unit.
tags: [calendar, model]
status: draft
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
---

# Parts of a calendar

- a **clock** it counts: an observer's τ, a system's SBT, or GCT (see [timeline and frames](/time/timeline-and-frames.md))
- an **epoch**: the start of its first defined era (see [eras and epoch](/calendar/eras-and-epoch.md)); time before the epoch is fully supported
- a **unit graph**: the units and how they relate (see [unit kinds](/calendar/unit-kinds.md))
- **parallel cycles**: independent counters that run alongside the unit tree (on Earth, the week; see [parallel cycles](/calendar/parallel-cycles.md))
- **rules**: adjustments to units' members (see [adjustment rules](/calendar/adjustment-rules.md))
- **eras**: how the top-level count is numbered and labelled
- **formats**: see [format templates](/formats/templates.md)

# The fundamental unit

Every calendar has exactly one **fundamental unit**: the smallest unit counted as a whole integer (on Earth, the day). All other units are subdivisions or groupings of it. It is either:

- **Fixed:** a constant duration on the calendar's clock (e.g. 88,775 s, or "the mean solar rotation fitted from simulation, frozen at epoch"; Earth's day is 86,400 s). Fast and exact.
- **Event-delimited:** each instance runs from one physical event to the next (e.g. sunrise to sunrise at site X), from the [event finder](/physics/event-finder.md). Lengths vary naturally.

Each instance gets an integer **fixed number** counted from the epoch: 0 is the first fundamental unit of the first era; earlier units are negative. It plays the part Earth's Julian Day Number plays for Earth's calendars, and the backbone every other unit is built on.
