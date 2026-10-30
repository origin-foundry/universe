---
type: Overview
title: Universe overview
description: Goals, non-goals and the overall shape of the Universe app.
tags: [overview, scope]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:09:07Z }
---

# Summary

Universe is a personal desktop app, developed in the open under GPL-3.0, for building fictional universes of star systems, simulating them, and defining calendars on top of the simulation that have no built-in notion of day, week, month or year.

This bundle is the long-lived source of truth for *what the system is*. Feature specs (under `specs/`) describe *what a change does*, and must cite the concepts here that they rely on or alter.

# Goals

- Let a user build a **universe**: several star systems, each with stars, planets and moons defined by masses and orbital elements. See [bodies](/physics/bodies.md).
- Simulate that universe with **N-body physics** over roughly ±100,000 years with second-level timing precision. See [integration](/physics/integration.md).
- Derive natural **cycles** (days, years, lunations, seasons, eclipses) from the simulation instead of typing them in. See [event finder](/physics/event-finder.md).
- Let the user define **calendars** with arbitrary nested units, parallel cycles, variable-length units, named members, intercalary units, eras and leap rules. See [calendar model](/calendar/calendar-model.md).
- Support three kinds of leap logic: [hand-authored](/calendar/leap-rules.md), [auto-suggested](/calendar/rule-suggestion.md) and [observational](/calendar/observational-calendars.md).
- Define **date formats** with conditional logic that can both render and parse. See [format templates](/formats/templates.md).
- Model **proper time**: observers on planets, in orbit, or on ships experience time at different rates, and each calendar is tied to a specific clock. See [proper time](/time/proper-time.md).
- Ship real-world presets that double as a correctness test suite. See [presets and validation](/product/presets-and-validation.md).

# Non-goals (for now)

- Full general relativity. Weak-field corrections in-system and special relativity between systems are used instead. See [ship trajectories](/time/ship-trajectories.md).
- Rigid-body spin dynamics. See [rotation](/physics/rotation.md).
- Stellar evolution, geologic or cosmological timescales.
- Collaboration, cloud sync, or binary releases. The source is public, but the app isn't supported for outside users. See [ADR 0004](/decisions/0004-public-gpl-repo.md).

# How work is done

Changes follow a spec-driven, two-PR workflow with human approval steps. See [ADR 0006](/decisions/0006-spec-driven-two-pr-workflow.md) and [ADR 0007](/decisions/0007-bot-identity-and-gates.md).
