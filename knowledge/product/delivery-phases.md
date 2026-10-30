---
type: Roadmap
title: Delivery phases
description: The six phases, each ending in something usable, with calendar core first.
tags: [roadmap, phases, planning]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:14:13Z }
---

# Phases

| Phase | Scope | Usable outcome |
|---|---|---|
| **0. Foundations** | Workflow setup first (see [ADR 0006](/decisions/0006-spec-driven-two-pr-workflow.md) and [ADR 0007](/decisions/0007-bot-identity-and-gates.md)): Git and the public GitHub repo, GPL-3.0 licence, bot account, ruleset and CODEOWNERS, Spec Kit and the constitution, `CLAUDE.md` and Claude Code hooks, CI checks, PR templates, project board. Then, as the first spec through that workflow: Rust workspace, `Instant` and rationals, SQLite schema and migrations, Tauri shell. | Every change is gated by the workflow; app opens, creates and saves an empty universe. |
| **1. Calendar core** | Fixed fundamental unit, unit tree, groupings, members, parallel cycles, arithmetic rules, multiple eras, epoch anchoring and pre-epoch dates, expression language + builder, formats (render and parse), converter, calendar grid. Gregorian/Julian/Islamic-tabular/Hebrew presets on a fixed clock. | Design and convert arithmetic calendars. |
| **2. Bodies & Kepler** | Body editor, orbital elements, Keplerian (two-body) propagation, spin models, event finder, derived cycles, 2D orrery, rule suggester, surface sites, time zone schemes and zone rules. | Calendars derived from a planet's orbit and rotation, with local time. |
| **3. N-body** | REBOUND binding (IAS15/WHFast), ephemeris cache, reliability indicator, observational calendars, event-delimited units, sky-based zone rules, drift charts, Solar System validation. | Realistic long-term drift and observational calendars. |
| **4. Proper time & multiple systems** | Galactic frame, SBT, surface and orbital observers, clock tables, calendars bound to clocks, converter showing proper-time differences. | Compare calendars across planets and systems. |
| **5. Ships** | In-system thrust trajectories, interstellar relativistic legs, handoff, ship clocks, light-delay queries, timeline view. | Model voyages and the time dilation they cause. |
| **6. Polish** | 3D orrery, JSON import/export, project compaction, performance, documentation, example fictional universes. | Shareable, presentable app. |

# Order

Calendar core comes first. See [ADR 0005](/decisions/0005-calendar-core-first.md).

Each phase is delivered as a series of small feature specs, not in one piece.
