---
type: Design
title: Background jobs
description: How long-running physics and table builds run without blocking the UI.
tags: [architecture, concurrency, jobs]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:12:40Z }
---

# Model

Long jobs run on a Rust worker pool in `uni-app` (see [crate layout](/architecture/crate-layout.md)):

- integrating an ephemeris ([integration](/physics/integration.md))
- building event or calendar tables ([event finder](/physics/event-finder.md), [conversion](/calendar/conversion.md))
- rule suggestion ([rule suggestion](/calendar/rule-suggestion.md))

Jobs report progress to the UI, can be cancelled, and write results to the cache tables (see [storage schema](/storage/schema.md)). The UI never blocks; views show "computing…" states for data that isn't ready.
