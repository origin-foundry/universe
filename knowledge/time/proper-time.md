---
type: Design
title: Proper time
description: How each observer's clock is derived from gravity and motion in-system.
tags: [time, relativity, observers, clocks]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:10:03Z }
---

# Weak-field formula

For an observer at position **x** with velocity **v** in the system barycentric frame:

```
dτ/dt ≈ 1 + Φ(x)/c² − v²/(2c²)
```

where Φ is the summed Newtonian potential of all bodies (negative), taken from the [N-body ephemeris](/physics/integration.md). This is the same first-order approximation used for real-world TT/TCB and GPS clock corrections, accurate to roughly 10⁻¹⁶ per unit time for planetary conditions.

When a ship's speed in-system exceeds ~0.01c, the app switches to the exact special-relativistic factor `sqrt(1 − v²/c²)` combined with the potential term, and warns that Newtonian trajectory dynamics are becoming approximate.

# Observers

| Kind | Path through the field |
|---|---|
| Surface site | Fixed latitude/longitude/altitude on a rotating body (see [rotation](/physics/rotation.md)). |
| Orbiting station | Its own orbit in the ephemeris. |
| Ship | A trajectory; see [ship trajectories](/time/ship-trajectories.md). |

Each observer's τ(t) is cached as a clock table (see [storage schema](/storage/schema.md)). Calendars bind to a clock; see [calendar model](/calendar/calendar-model.md).

# Validation

GPS-orbit clock rate offset (~+38 µs/day) is a required test case. See [presets and validation](/product/presets-and-validation.md).
