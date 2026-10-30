---
type: Glossary
title: Glossary
description: Defined terms used across the Universe design.
tags: [glossary]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:09:07Z }
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
| **Cycle** | A recurring physical phenomenon detected from the simulation (e.g. "local noon at site X"). See [event finder](/physics/event-finder.md). |
| **Calendar** | A naming scheme for instants, bound to one clock, made of units, cycles, rules and eras. Its epoch is the start of its first era. See [calendar model](/calendar/calendar-model.md). |
| **Fundamental unit** | The smallest whole-number-counted unit of a calendar (Earth's day). |
| **Fixed number** | The integer index of a fundamental unit, counted from the calendar epoch (0 = first unit of the first era). |
| **Unit** | A named division of time inside a calendar. See [unit kinds](/calendar/unit-kinds.md). |
| **Parallel cycle** | A modular counter running alongside the unit tree without nesting (e.g. a week). See [parallel cycles](/calendar/parallel-cycles.md). |
| **Time zone scheme** | How local time is reckoned on a body. See [time zones](/calendar/time-zones.md). |
| **Format** | A template that renders a date as text and parses text back. See [format templates](/formats/templates.md). |
| **Event** | A named instant or interval on the absolute timeline, shown in every calendar. |

# The two questions a date answers

1. **Where is this on the absolute timeline?** A GCT instant, used for "what was happening elsewhere at the same moment".
2. **How much time did this observer live through?** A proper-time value on that observer's clock.

A traveller's calendar and the home planet's calendar can name the same GCT instant as dates years apart, and both are correct. The converter shows both.

Simultaneity is a convention: "the same moment on two planets" means "the same GCT". [Light-delay queries](/time/light-delay.md) answer the separate question "when could B have learned about A?"
