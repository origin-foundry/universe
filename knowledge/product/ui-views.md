---
type: Design
title: UI views
description: The app's editors and views and what each is for.
tags: [ui, product]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:14:13Z }
---

# Views

| View | Purpose |
|---|---|
| **Universe editor** | Systems, [bodies](/physics/bodies.md) (hierarchical tree), orbital elements, spin models, galactic placement. A live mini-orrery previews edits. |
| **Time zones** | Per body: pick a [scheme](/calendar/time-zones.md), draw zones on a longitude map, edit transitions and seasonal rules, preview a zone's offset over time. |
| **Observers & ships** | Surface sites (map picker on the body), stations, ships with a segment-by-segment [trajectory](/time/ship-trajectories.md) editor and a plot of the onboard clock vs. absolute time. |
| **Calendar editor** | Unit tree, members, parallel cycles, rules (builder + advanced), eras and pre-epoch labelling, epoch anchoring, locales. Live preview: a calendar grid plus drift statistics. |
| **Rule suggester** | Pick a target cycle; view candidate rules on an accuracy-vs-simplicity chart; apply one. See [rule suggestion](/calendar/rule-suggestion.md). |
| **Format editor** | Template with syntax highlighting, live render, parse test box and round-trip report. See [format parsing](/formats/parsing.md). |
| **Converter** | Enter a date in any calendar/format. See it in every calendar and every observer's clock side by side, with proper-time differences called out. Never shows raw GCT (see [display units](/time/display-units.md)). |
| **Calendar grid** | Page-style view of any unit (e.g. a "month" equivalent) with festivals, moon phases, seasons and eclipses marked. |
| **Orrery** | 2D and 3D views of a system, scrubbable through time, with body trails and ship trajectories. Clicking a moment shows it in the converter. |
| **Timeline** | Events across time in parallel lanes per calendar. Includes **drift charts**: how far a calendar's units wander from the sky events they track. |
