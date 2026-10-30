---
type: Architecture Decision
title: "ADR 0002: Hidden absolute timeline"
description: GCT is a hidden technical zero; each calendar's epoch is the start of its first era.
tags: [decision, time, calendar, epoch]
status: stable
decided: 2026-10-29
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:14:45Z }
---

# Context

Calendars need a shared reference to be comparable, but a named universal epoch would impose a meaning users don't want.

# Decision

- The absolute timeline (GCT) uses a **hidden technical zero** and is never shown to users. It exists only to equate points on different calendars. See [timeline and frames](/time/timeline-and-frames.md).
- Calendars can have **multiple eras**. The **first era's start is the calendar's epoch**. See [eras and epoch](/calendar/eras-and-epoch.md).
- **Dates before the epoch** are fully supported (proleptic), labelled by a pre-epoch era and/or astronomical numbering.

# Consequences

- The UI must always express time in a user calendar; a debug-only GCT display is allowed.
- Epochs are anchored by equating dates across calendars, or to timeline or sky events.
