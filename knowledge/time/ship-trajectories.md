---
type: Design
title: Ship trajectories
description: The hybrid in-system and interstellar model for ships and their clocks.
tags: [time, relativity, ships, trajectories]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:10:03Z }
---

# Hybrid model

Ships fly full trajectories. The model is hybrid, honest about where each piece of physics is valid:

| Segment | Dynamics | Clock |
|---|---|---|
| **In-system** (departure, arrival, manoeuvres) | Test particle integrated inside the system's [N-body simulation](/physics/integration.md), with a thrust schedule (burn start, duration, direction, acceleration). Thrust is applied via REBOUND's `additional_forces` callback. | Weak-field formula; see [proper time](/time/proper-time.md). |
| **Interstellar** | Special-relativistic motion in the galactic frame: constant proper acceleration ("hyperbolic motion"), coast and deceleration phases, solved in closed form. | Exact: e.g. `t = (c/a)·sinh(aτ/c)` during constant acceleration. |
| **Handoff** | At a configurable distance from a system's barycentre, state vectors are converted between the system frame and the galactic frame (Lorentz boost by the system's velocity). | Continuous across the boundary. |

# Limits

Fully self-consistent relativistic N-body dynamics is out of scope. For any speed and acceleration a fictional ship will plausibly use, the hybrid model agrees with it to well below a second over a voyage.

# Validation

Twin-paradox voyages are checked against closed-form answers. See [presets and validation](/product/presets-and-validation.md).
