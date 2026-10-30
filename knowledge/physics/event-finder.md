---
type: Design
title: Event finder
description: Root-finding on the ephemeris to detect sky events and fit mean periods.
tags: [physics, events, cycles]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:02Z }
---

# Method

Derived cycles come from root-finding on the [ephemeris](/physics/integration.md): bracketing plus Brent's method.

# Built-in event types

| Event | Definition |
|---|---|
| Transit / local noon & midnight | A star crosses the meridian of a surface site. |
| Rise / set | A body's centre or limb crosses the horizon (with optional refraction). |
| Equinox / solstice | The star's declination as seen from a planet crosses zero or reaches an extremum. |
| Periapsis / apoapsis | A body's distance to its parent reaches a minimum or maximum. |
| Conjunction / opposition / phase | Elongation between two bodies as seen from a third reaches a given value (e.g. new moon). |
| Eclipse | Shadow geometry overlap. |
| Custom | Any [expression](/language/expression-language.md) over ephemeris quantities crossing a threshold. |

# Mean periods

**Mean periods** are fitted over a user-chosen window (e.g. "mean tropical year over ±1,000 years"), because N-body periods aren't constant. Both the event list and the mean value are available to calendars: mean values feed [rule suggestion](/calendar/rule-suggestion.md); event lists feed [observational calendars](/calendar/observational-calendars.md).

Results are cached in `event_cache`; see the [storage schema](/storage/schema.md).
