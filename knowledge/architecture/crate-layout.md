---
type: Design
title: Crate layout
description: The Rust workspace crates and their dependency direction.
tags: [architecture, rust, crates, layering]
status: draft
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
---

# Layout

```
Cargo.toml        workspace root (members: crates/*)
package.json      UI dependencies and the Tauri CLI; npm scripts dev and app
crates/
  uni-time/       Instant, rationals, frame chain (GCT/SBT/τ), clock tables
  rebound-sys/    vendored REBOUND C source, build.rs, bindgen FFI
  uni-physics/    bodies, element conversion, safe REBOUND wrapper (Integrator trait), spin, event finder, ephemeris cache
  uni-zones/      time zone schemes, transitions, local-time resolution
  uni-expr/       lexer, parser, type checker, evaluator for the expression language
  uni-calendar/   unit graph, rules, cycles, eras, conversion strategies, rule suggester
  uni-format/     template compiler, renderer, parser, locales
  uni-store/      SQLite schema, migrations, repositories, cache management
  uni-presets/    Solar System + Earth calendars
  uni-app/        the Tauri app: tauri.conf.json, Tauri commands, background job runner, progress events
ui/               React app sources (built by Vite from the root package.json)
```

# Rules

- Dependencies only point downward. The allowed dependencies below are the complete list: a dependency between workspace crates that isn't listed is not allowed.
- Core crates have no Tauri dependency, so they can later be used as a library or CLI. Only `uni-app` uses Tauri.
- Calendar crates use exact integer/rational arithmetic only, never `f64`; see [timeline and frames](/time/timeline-and-frames.md). The crates listed under `exact` may not use floating-point types or literals at all.
- The expression language's sky functions are an interface defined in `uni-expr` and implemented in `uni-physics`. That's how expressions (and zone rules) can ask about the sky without `uni-expr` depending on physics, which would create a cycle with physics' custom events.
- Only `uni-physics` uses `rebound-sys`.

# Allowed dependencies

The architecture check (`scripts/check_architecture.py`) reads this block and fails any PR whose crates don't match it. Changing it needs a design PR.

```yaml crate-dependencies
uni-time: []
rebound-sys: []
uni-expr: [uni-time]
uni-zones: [uni-time, uni-expr]
uni-calendar: [uni-time, uni-expr, uni-zones]
uni-format: [uni-time, uni-expr, uni-zones, uni-calendar]
uni-physics: [uni-time, uni-expr, rebound-sys]
uni-store: [uni-time, uni-expr, uni-zones, uni-calendar, uni-format, uni-physics]
uni-presets: [uni-time, uni-expr, uni-zones, uni-calendar, uni-format, uni-physics]
uni-app: [uni-time, uni-expr, uni-zones, uni-calendar, uni-format, uni-physics, uni-store, uni-presets]
exact: [uni-calendar, uni-expr, uni-format, uni-zones]
```

Related: [stack](/architecture/stack.md), [background jobs](/architecture/background-jobs.md).
