---
type: Design
title: Parallel cycles
description: Modular counters such as weeks that run alongside the unit tree.
tags: [calendar, cycles, weeks]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:22Z }
---

# Definition

A parallel cycle is a modular counter over some unit, with:

- period *n* and optional named members
- an **anchor**: which member falls on a given fixed number
- an optional **skip rule**: units the cycle doesn't advance on (for [intercalary members](/calendar/unit-kinds.md))

Multiple parallel cycles can coexist. This covers the 7-day week, market weeks, and Maya-style interlocking counts (Tzolk'in 13 × 20 against Haab' 365).

Skip rules are written in the [expression language](/language/expression-language.md).
