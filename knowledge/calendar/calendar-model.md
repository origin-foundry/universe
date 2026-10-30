---
type: Design
title: Calendar model
description: What a calendar is made of, its clock, and the fundamental unit.
tags: [calendar, model]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:22Z }
---

# Parts of a calendar

- a **clock** it counts: an observer's τ, a system's SBT, or GCT (see [timeline and frames](/time/timeline-and-frames.md))
- an **epoch**: the start of its first defined era (see [eras and epoch](/calendar/eras-and-epoch.md)); time before the epoch is fully supported
- a **unit graph**: the units and how they relate (see [unit kinds](/calendar/unit-kinds.md))
- **parallel cycles**: independent counters such as weeks (see [parallel cycles](/calendar/parallel-cycles.md))
- **rules**: leap and adjustment logic (see [leap rules](/calendar/leap-rules.md))
- **eras**: how the top-level count is numbered and labelled
- **formats**: see [format templates](/formats/templates.md)

# The fundamental unit

Every calendar has exactly one **fundamental unit**: the smallest unit counted as a whole integer (on Earth, the day). All other units are subdivisions or groupings of it. It is either:

- **Fixed:** a constant duration on the calendar's clock (e.g. 86,400 s, or "the mean solar day fitted from simulation, frozen at epoch"). Fast and exact.
- **Event-delimited:** each instance runs from one physical event to the next (e.g. sunrise to sunrise at site X), from the [event finder](/physics/event-finder.md). Lengths vary naturally.

Each instance gets an integer **fixed number** counted from the epoch: 0 is the first fundamental unit of the first era; earlier units are negative. It is the calendar's equivalent of the Julian Day Number, and the backbone every other unit is built on.
