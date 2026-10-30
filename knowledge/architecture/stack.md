---
type: Design
title: Stack
description: Tauri 2 with a Rust core and a TypeScript/React UI.
tags: [architecture, stack, tauri, rust, react]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:12:40Z }
---

# Choice

**Tauri 2** desktop app:

- **Rust core**, for numerically heavy physics and exact calendar arithmetic, with predictable performance.
- **TypeScript + React UI.** **react-three-fiber** (Three.js) for the orrery, **visx/d3** for timelines and drift charts.
- **rusqlite** for storage; see [storage schema](/storage/schema.md).
- **REBOUND** (C) for N-body integration, via FFI; see [ADR 0001](/decisions/0001-bind-rebound.md).

# Rationale

The user asked for a desktop app and left the stack open. Rust suits the precision and performance needs of [integration](/physics/integration.md) and [proper time](/time/proper-time.md); a web UI makes the orrery and calendar grids straightforward.

See [crate layout](/architecture/crate-layout.md).
