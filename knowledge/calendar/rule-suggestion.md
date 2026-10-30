---
type: Design
title: Rule suggestion
description: Proposing leap rules from simulated periods via continued fractions.
tags: [calendar, leap-rules, suggestion]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:22Z }
---

# Algorithm

Given a target ratio from the simulation (e.g. mean tropical year ÷ mean solar day = 412.3719…, from the [event finder](/physics/event-finder.md)), the suggester:

1. Computes **continued-fraction convergents** (e.g. 412/1, 4123/10, 21031/51 …), giving candidate cycle lengths.
2. Converts promising candidates into **nested rule sets** ("leap every 3rd, except every 51st, …") by greedy decomposition.
3. Scores each option on **drift per millennium** against both the mean period and the actual simulated event series, plus **complexity** (number of rule clauses).
4. Shows the results as a Pareto chart (accuracy vs. simplicity). The user picks one, then edits it like any hand-authored [leap rule](/calendar/leap-rules.md).

# UI

See the rule suggester view in [UI views](/product/ui-views.md).
