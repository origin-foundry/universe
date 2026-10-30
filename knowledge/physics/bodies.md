---
type: Design
title: Bodies
description: How stars, planets and moons are defined, and how systems sit in the galactic frame.
tags: [physics, bodies, orbits]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:02Z }
---

# Body definition

Each body has:

- mass, radius, optional oblateness (J2, for precession effects)
- a parent body (star → planet → moon), forming a hierarchy
- **osculating orbital elements** relative to its parent at the system epoch: semi-major axis *a*, eccentricity *e*, inclination *i*, longitude of ascending node Ω, argument of periapsis ω, mean anomaly *M*
- a spin model; see [rotation](/physics/rotation.md)

Elements are converted to Cartesian state vectors in **Jacobi coordinates**, so hierarchical systems (binary stars, moons of moons) start in sensible configurations. Advanced users can enter state vectors directly.

# Systems in the galactic frame

Systems have a galactic position and velocity (straight-line motion; galactic orbit ignored over ±100k years). The velocity sets the GCT → SBT rate; see [timeline and frames](/time/timeline-and-frames.md).

# Storage

See the `system`, `body`, `body_elements` and `spin_model` tables in the [storage schema](/storage/schema.md).
