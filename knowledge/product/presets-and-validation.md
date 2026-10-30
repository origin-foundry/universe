---
type: Test Strategy
title: Presets and validation
description: Real-world presets that double as the correctness test oracle.
tags: [testing, presets, validation, earth]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:14:13Z }
---

# Presets

| Preset | Validates |
|---|---|
| **Solar System** (Sun, eight planets, major moons; J2000 mean elements) | [Integration](/physics/integration.md) accuracy: positions against published ephemerides within stated tolerances over ±a few thousand years; derived tropical year ≈ 365.2422 days; synodic month ≈ 29.5306 days. |
| **Gregorian & Julian** | Arithmetic grouping rules, supercycle detection, eras without year zero, weeks. See the [Gregorian example](/calendar/gregorian-example.md). |
| **Islamic (tabular)** | Rules on a position within a 30-year cycle. |
| **Islamic (observational)** | [Observational months](/calendar/observational-calendars.md) from crescent visibility, compared against published visibility predictions (approximate). |
| **Hebrew** | Length/start expressions, molad arithmetic, postponement rules, leap months. See [leap rules](/calendar/leap-rules.md). |
| **Proper time** | GPS-orbit clock rate offset (~+38 µs/day); twin-paradox voyages against closed-form answers. See [proper time](/time/proper-time.md). |

# Automated tests

- Hundreds of known date conversions across these calendars. The expected values are **generated from an independent open-source implementation** (not copied from published reference tables, which are copyrighted). The generator script, the library and its version are committed alongside the test data, so the data can be regenerated and its origin checked.
- **Property tests:** date → instant → date round-trips for every calendar, and format render → parse round-trips (see [format parsing](/formats/parsing.md)).

# Limits

JPL's high-precision ephemerides aren't bundled. The Solar System preset uses mean elements plus REBOUND, so it's "accurate enough to reproduce real calendars", not navigation-grade.
