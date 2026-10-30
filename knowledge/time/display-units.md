---
type: Design
title: Display units
description: SI internally; the UI shows only the user's own units.
tags: [time, units, ui]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:10:03Z }
---

# Rule

All physics and storage use SI units (seconds, metres, kilograms). The UI never shows Earth units or raw GCT values unless the user asks.

- Each universe has a **display calendar** (user-chosen) used to show durations and dates by default.
- Durations are shown in that calendar's units ("3 cycles, 14 tenths"), with an optional "show SI" toggle for debugging and for the Earth presets. See [duration formats](/formats/templates.md).
- Physical inputs (mass, distance) can be entered in user-defined units (e.g. "1 home-mass = 5.2 × 10²⁴ kg") or in SI.

Related: [timeline and frames](/time/timeline-and-frames.md), [ADR 0002](/decisions/0002-hidden-absolute-timeline.md).
