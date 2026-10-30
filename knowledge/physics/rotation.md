---
type: Design
title: Rotation
description: Kinematic spin models with secular drift, tilt and precession.
tags: [physics, rotation, spin]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:02Z }
---

# Spin model

Spin isn't produced by translational N-body integration, so each [body](/physics/bodies.md) has a kinematic spin model:

- sidereal rotation period at epoch, plus a **secular drift** term (e.g. tidal braking, in seconds of period per century)
- axial tilt (obliquity) and pole direction
- axial **precession** rate and optional nutation terms
- option to **tidally lock** to the parent (rotation derived from the orbit)

Rigid-body spin dynamics is a non-goal; see the [overview](/overview.md).

# Uses

Rotation drives surface-site positions (for [proper time](/time/proper-time.md) and [time zones](/calendar/time-zones.md)) and transit, rise and set events in the [event finder](/physics/event-finder.md).
