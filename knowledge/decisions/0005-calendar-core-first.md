---
type: Architecture Decision
title: "ADR 0005: Calendar core first"
description: Build the calendar engine before any physics.
tags: [decision, roadmap]
status: stable
decided: 2026-10-29
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:14:45Z }
---

# Context

The user left the build order to Claude.

# Decision

Build the **calendar core first** (phase 1), then physics. See [delivery phases](/product/delivery-phases.md).

# Rationale

- The calendar engine has no dependency on physics.
- Every physics feature (derived cycles, observational units, rule suggestion, drift charts) needs a calendar engine to be useful.
- The Earth presets give the engine a strong test oracle from day one (see [presets and validation](/product/presets-and-validation.md)), so foundations are verified before the harder numerical work.
