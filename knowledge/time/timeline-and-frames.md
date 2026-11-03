---
type: Design
title: Timeline and frames
description: The hidden absolute timeline (GCT), system time (SBT), and how instants are represented.
tags: [time, gct, sbt, frames, precision]
status: draft
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
---

# Absolute timeline

Every instant in the app is ultimately a value of **Galactic Coordinate Time (GCT)**: SI seconds in the single inertial galactic frame.

GCT zero is a **hidden technical zero**, chosen by the app when a universe is created (in practice the physics epoch of the first system). It has no name and no meaning to the user. GCT exists only so points on different calendars can be equated; every date shown is expressed in some calendar. A "Show GCT" option exists only in a developer/debug panel. See [ADR 0002](/decisions/0002-hidden-absolute-timeline.md).

# Representation

| Domain | Type | Why |
|---|---|---|
| Absolute instants (GCT, SBT, τ) | `Instant { secs: i64, nanos: u32 }` | Exact, ordered, ±292 billion years range, nanosecond resolution. |
| Calendar arithmetic | Exact rationals (`i128 / i128`) and integers | Leap rules and unit lengths must never accumulate float error. |
| Physics integration | `f64` seconds relative to a simulation epoch near the middle of the range | At ±3.2 × 10¹² s (100k years), f64 resolution is ~0.5 ms, comfortably within the second-precision target. |

Conversions between `Instant` and physics `f64` happen at one well-tested boundary, in `uni-physics`. `uni-time` itself is an exact crate with no floating point (see [crate layout](/architecture/crate-layout.md)).

# Frame chain

```
GCT  ──(linear: rate + offset per system)──▶  SBT_system
SBT  ──(integrated: dτ/dt along worldline)──▶  τ_observer
```

- **GCT → SBT** is linear. The rate comes from the system's velocity in the galactic frame (Lorentz factor) and an optional constant gravitational potential term. Users can override it.
- **SBT → τ** is integrated along the observer's path through the system's simulated gravitational field. See [proper time](/time/proper-time.md). The result is cached as a monotonic table, so the inverse (τ → SBT) is a binary search plus interpolation.

# Simultaneity

"The same moment on two planets" is defined as "the same GCT". This is a choice of frame, stated openly in the UI. See the [glossary](/glossary.md) for the two questions every date answers.
