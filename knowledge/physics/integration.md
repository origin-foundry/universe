---
type: Design
title: Integration
description: N-body simulation via REBOUND, the ephemeris cache and the reliability indicator.
tags: [physics, n-body, rebound, ephemeris, cache]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:02Z }
---

# Integrator

**REBOUND** (C library), bound into Rust via FFI (`bindgen` + a thin safe wrapper). See [ADR 0001](/decisions/0001-bind-rebound.md).

- **IAS15** (adaptive 15th-order Gauss–Radau) is the default, for accuracy.
- **WHFast** (Wisdom–Holman symplectic) is a fast option for long, well-behaved runs.
- REBOUND is vendored and compiled by `build.rs` with the `cc` crate, so no separate install is needed.
- The rest of the app talks to it through an `Integrator` trait, so it stays replaceable.
- Ship thrust uses REBOUND's `additional_forces` callback; see [ship trajectories](/time/ship-trajectories.md).

# Ephemeris cache

The simulation is deterministic given its inputs. Results are stored as:

- **checkpoints**: full state every N steps
- **dense output**: Hermite-interpolable state at a finer interval

Queries interpolate. Queries outside the cached span extend the integration forward or backward from the nearest checkpoint.

The **cache key** is a hash of every input that affects physics. Editing a body invalidates only that system's cache. See the cache tables in the [storage schema](/storage/schema.md).

# Chaos and reliability

N-body systems are chaotic over long spans. The app shows a **reliability indicator**, from comparing two runs with tiny perturbations, so users know when far-future dates stop being meaningful.

# Precision

Physics runs in `f64` seconds relative to a mid-range simulation epoch; see [timeline and frames](/time/timeline-and-frames.md).
