---
type: Architecture Decision
title: "ADR 0001: Bind REBOUND"
description: Use the REBOUND C library via FFI for N-body integration.
tags: [decision, physics, rebound]
status: stable
decided: 2026-10-29
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:14:45Z }
---

# Context

The app needs accurate N-body integration over ±100k years. Options were writing IAS15/WHFast natively in Rust, or binding the mature REBOUND C library.

# Decision

**Bind REBOUND** via FFI: vendored source, compiled by `build.rs` with the `cc` crate, `bindgen` bindings, wrapped behind an `Integrator` trait. See [integration](/physics/integration.md).

# Consequences

- Proven integrators immediately; much less numerical code to write and verify.
- A C build step on each platform. Windows/MSVC is a known risk; see [risks](/risks.md).
- REBOUND is GPL-3.0. The repository is GPL-3.0 too, so vendoring it is clean; see [ADR 0004](/decisions/0004-public-gpl-repo.md).
- The trait keeps a native Rust integrator possible later.
