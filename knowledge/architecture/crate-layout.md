---
type: Design
title: Crate layout
description: The Rust workspace crates and their dependency direction.
tags: [architecture, rust, crates, layering]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:12:40Z }
---

# Layout

```
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
  uni-app/        Tauri commands, background job runner, progress events
ui/               React app
```

# Rules

- Dependencies only point downward: `uni-calendar` uses `uni-expr` and `uni-time`; `uni-physics` never depends on calendars.
- Core crates have no Tauri dependency, so they can later be used as a library or CLI.
- Calendar crates use exact integer/rational arithmetic only, never `f64`; see [timeline and frames](/time/timeline-and-frames.md).

Related: [stack](/architecture/stack.md), [background jobs](/architecture/background-jobs.md).
