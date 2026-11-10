---
type: Architecture Decision
title: "ADR 0008: Physical time and calendar time"
description: Physics works in physical time on GCT; calendars are human conventions mapped onto it and don't have to be astronomically accurate.
tags: [decision, time, calendar, physics, terminology]
status: stable
decided: 2026-11-10
generated: { by: claude-code/claude-opus-5-5, at: 2026-11-10T12:00:00Z }
verified: { by: human:origin-foundry, at: 2026-11-10T22:41:49Z }
---

# Context

Universe models true astronomical time with physics, while the calendars people define rarely match the sky exactly: a calendar's day may differ from the planet's real rotation, and real rotations change over time. Without a clear line between the two, later features could treat a calendar's "day" as the physical day, refuse calendars that drift from the sky, or use the same word for both. This came up in the review of spec 004 (calendar engine basics).

# Decision

These ground rules hold for every feature. Each spec states which side of the line each concept it uses is on.

1. **Physical time** is GCT and the clocks derived from it (system time, an observer's proper time). Physics works only in these. See [timeline and frames](/time/timeline-and-frames.md).
2. **Real phenomena** (a planet's rotation, its orbit, sunrise, a lunation) are events and **physical cycles** that the physics finds and places on the timeline. They are facts. See [event finder](/physics/event-finder.md).
3. **A calendar is a human convention** laid over one clock, tied to the timeline only by that clock and its epoch anchor. It never alters physics, and physics never depends on it. See [calendar model](/calendar/calendar-model.md) and [crate layout](/architecture/crate-layout.md).
4. **Calendars don't have to be astronomically accurate.** A calendar's day, month or year is whatever its definer chooses. Matching the sky is optional: a unit can be any fixed length (perhaps fitted from physics as a starting point, then frozen), rule-adjusted ([leap rules](/calendar/leap-rules.md)), or event-delimited ([observational calendars](/calendar/observational-calendars.md)), and the first is as valid as the last.
5. **Drift between a calendar and the sky is never an error.** The engine never refuses or warns about it. When physics exists, it is shown as information (drift charts, the [rule suggester](/calendar/rule-suggestion.md)) for users who care.
6. **The app says so plainly.** When calendar editing reaches the UI, it tells the user that units are their choice and needn't match the sky, and never presents astronomical accuracy as a requirement.
7. **Words.** "Day", "year" and so on inside a calendar mean calendar units; physical ones are always named as such ("rotation", "solar day at site X"). A recurring phenomenon is a **physical cycle**, never just a "cycle", so it can't be confused with a calendar's [parallel cycle](/calendar/parallel-cycles.md). "Astronomical year numbering" (year 0, −1, …) is only a numbering convention, not physics. See the [glossary](/glossary.md).

# Consequences

- The calendar engine accepts any positive unit length and never compares it with physics.
- Physics-derived features (fitting a unit from simulation, drift charts, rule suggestion, event-delimited units) are options offered to the user, never checks imposed on calendars.
- Specs, concepts and code use "physical cycle", "rotation" and similar names for physical phenomena, and calendar unit names only for calendar units.
- Builds on [ADR 0002](/decisions/0002-hidden-absolute-timeline.md) (the hidden timeline every calendar is mapped onto) and [ADR 0005](/decisions/0005-calendar-core-first.md) (the calendar engine stands on its own before physics).
