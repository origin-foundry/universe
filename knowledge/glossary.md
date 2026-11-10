---
type: Glossary
title: Glossary
description: Defined terms used across the Universe design.
tags: [glossary]
status: draft
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
---

# Terms

| Term | Meaning |
|---|---|
| **Universe** | One project. Contains systems, observers, calendars, formats and events. |
| **Galactic frame** | The single inertial reference frame the whole universe is described in. Its coordinate time is the absolute timeline. |
| **GCT** | *Galactic Coordinate Time*. The absolute timeline, in SI seconds from a hidden technical zero. Never shown to users. See [timeline and frames](/time/timeline-and-frames.md). |
| **System** | A star system: a set of gravitationally bound bodies with a position and velocity in the galactic frame. |
| **SBT** | *System Barycentric Time*. Coordinate time at a system's barycentre, related to GCT by a fixed rate and offset. |
| **Body** | A star, planet, moon or other massive object inside a system. See [bodies](/physics/bodies.md). |
| **Observer** | Something that experiences time: a site on a body's surface, an orbiting station, or a ship. |
| **Clock** | The proper time τ experienced by an observer, as a function of GCT. See [proper time](/time/proper-time.md). |
| **Physical time** | GCT and the clocks derived from it (SBT, an observer's proper time). The only time physics works in. See [ADR 0008](/decisions/0008-physical-time-and-calendar-time.md). |
| **Physical cycle** | A recurring physical phenomenon detected from the simulation (e.g. "local noon at site X", a rotation, a lunation). A fact, not a calendar unit; never called just a "cycle". See [event finder](/physics/event-finder.md). |
| **Calendar** | A naming scheme for instants, bound to one clock, made of units, parallel cycles, rules and eras. Its epoch is the start of its first era. A human convention: it doesn't have to match the sky. See [calendar model](/calendar/calendar-model.md). |
| **Calendar time** | Time named by a calendar's units (its "day", "year"). Mapped onto physical time by the calendar's clock and epoch. See [ADR 0008](/decisions/0008-physical-time-and-calendar-time.md). |
| **Fundamental unit** | The smallest whole-number-counted unit of a calendar (Earth's day). |
| **Fixed number** | The integer index of a fundamental unit, counted from the calendar epoch (0 = first unit of the first era). |
| **Unit** | A named division of time inside a calendar. See [unit kinds](/calendar/unit-kinds.md). |
| **Member unit** | The unit a grouping's members are instances of (a year's months). See [unit kinds](/calendar/unit-kinds.md). |
| **First number** | The number of a unit's first instance within its parent (1 for months and days, 0 for hours by default). See [unit kinds](/calendar/unit-kinds.md). |
| **Era's unit** | The unit an era numbers, usually the year; units above it are numbered by count from the epoch. See [eras and epoch](/calendar/eras-and-epoch.md). |
| **Astronomical year numbering** | Numbering that runs 1, 0, −1, −2 … across the epoch with no gap. A numbering convention, not physics. See [eras and epoch](/calendar/eras-and-epoch.md). |
| **Parallel cycle** | A modular counter running alongside the unit tree without nesting (e.g. a week). See [parallel cycles](/calendar/parallel-cycles.md). |
| **Time zone scheme** | How local time is reckoned on a body. See [time zones](/calendar/time-zones.md). |
| **Format** | A template that renders a date as text and parses text back. See [format templates](/formats/templates.md). |
| **Event** | A named instant or interval on the absolute timeline, shown in every calendar. |

# The two questions a date answers

1. **Where is this on the absolute timeline?** A GCT instant, used for "what was happening elsewhere at the same moment".
2. **How much time did this observer live through?** A proper-time value on that observer's clock.

A traveller's calendar and the home planet's calendar can name the same GCT instant as dates years apart, and both are correct. The converter shows both.

Simultaneity is a convention: "the same moment on two planets" means "the same GCT". [Light-delay queries](/time/light-delay.md) answer the separate question "when could B have learned about A?"
