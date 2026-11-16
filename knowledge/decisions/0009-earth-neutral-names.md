---
type: Architecture Decision
title: "ADR 0009: Earth-neutral names"
description: Code and design use the calendar model's own terms, not Earth's (year, month, leap); Earth words appear only where the subject really is Earth, and a check enforces it in code.
tags: [decision, terminology, calendar, code]
status: stable
decided: 2026-11-16
generated: { by: claude-code/claude-opus-5-5, at: 2026-11-16T16:00:00Z }
verified: { by: human:origin-foundry, at: 2026-11-16T18:19:49Z }
---

# Context

Universe defines calendars for worlds that aren't Earth. Its design already has general terms for every part of a calendar ([glossary](/glossary.md), [unit kinds](/calendar/unit-kinds.md)): the fundamental unit, groupings and their members, member units, subdivisions, the era's unit, parallel cycles. But the first engine's code, tests and parts of this bundle drifted into Earth's vocabulary: variables called `year` and `months`, error fields called `year`, a type called `LeapRuleDef`, test names like `convert_century_years`, and a non-Earth test calendar (Aster) whose units were called year, month, hour and minute.

That vocabulary quietly assumes Earth's shape: that the era's unit is an orbit, that there are months inside it, that a calendar has a day. None of these is true of every calendar Universe must handle, and code that thinks in Earth terms makes Earth-shaped mistakes. The words a user sees already come from the definer's own unit names, so this is about the code and the design, not the product's text.

# Decision

1. **Code uses the model's terms.** Identifiers, comments, strings, test names and test data say what the model says:

   | Instead of | Use |
   |---|---|
   | year | the era's unit; an instance of it; its number (`era_number`) |
   | month, months | member, members; member unit |
   | day | fundamental unit |
   | hour, minute | subdivision |
   | decade, century | a unit above the era's unit |
   | week | parallel cycle |
   | leap rule | adjustment rule |

2. **Earth words only where the subject is Earth.** They are allowed:
   - in code that is specifically about Earth's calendar, such as the Gregorian-shaped test calendar, its fixture, and later a Gregorian preset;
   - as an aside that says it is about Earth, where that helps a reader ("the era's unit; on Earth, the year").

3. **Non-Earth test calendars use invented names.** Aster's units are age, cohort, orbit, span, sol, bell and pulse (Tessel already has a watch); a stand-in for a parallel cycle is a rota. The test calendars inside unit tests use the same names.

4. **Adjustment rules.** "Leap rule" becomes **adjustment rule**: it adjusts a member's length, longer or shorter. The concept is renamed [adjustment rules](/calendar/adjustment-rules.md) and the type `AdjustmentRuleDef`. The error fields that held "the year" become `era_number`.

5. **The design follows the same rule.** This bundle uses the model's terms, with Earth as labelled examples (the "Earth example" columns, the [Gregorian example](/calendar/gregorian-example.md), "on Earth, …" asides). Finished specs (002 to 005, and spec 006 apart from its contract and data model, which describe the code's current names) stay as they were written: they are a record of what was decided then.

6. **A check enforces it.** `scripts/check_earth_terms.py` finds Earth words in the code, including inside identifiers (`year_def`, `LeapRuleDef`), and runs in CI. It allows files whose path contains "gregorian", lines that also say "Earth" or "Gregorian", and blocks between `earth-terms: off` and `earth-terms: on`. It is a guardrail, so loosening it needs a design PR. "Second" is not an Earth word: it is the SI unit physics uses ([ADR 0008](/decisions/0008-physical-time-and-calendar-time.md)).

# Consequences

- The public names in `uni-calendar` change (`AdjustmentRuleDef`, `era_number` fields), so spec 006's contract and data model change with them. Nothing outside the workspace uses them yet.
- Names can get a little longer (`era_number` rather than `year`), in exchange for code that doesn't suggest every calendar is Earth-shaped.
- A definer may still call their units year and month: unit names are data. The rule is about what the engine and its tests assume, not what users may choose.
