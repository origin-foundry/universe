---
type: Architecture Decision
title: "ADR 0003: Per-body time zones"
description: Each body picks its own time zone scheme, and schemes can be mixed.
tags: [decision, calendar, time-zones]
status: stable
decided: 2026-10-29
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:14:45Z }
---

# Context

Some worlds need full historical and seasonal zone rules; others need only per-site local time.

# Decision

Support **all schemes, chosen per body**: planetary standard, per-site solar time, named zones, and zones with rules. Schemes can be mixed on one body. See [time zones](/calendar/time-zones.md).

# Consequences

- A dedicated `uni-zones` crate; see [crate layout](/architecture/crate-layout.md).
- Ambiguous and skipped local times must be surfaced, never silently resolved.
- Zone work lands in phases 2 and 3; see [delivery phases](/product/delivery-phases.md).
