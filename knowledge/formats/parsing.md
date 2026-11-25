---
type: Design
title: Format parsing
description: Compiling templates into parsers, invertibility and ambiguity.
tags: [formats, parsing]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-11-25T20:25:13Z }
---

# Rules

Each [format template](/formats/templates.md) compiles into a parser as well as a renderer.

- Field tokens and named members parse naturally. Names match case-insensitively, by full name or abbreviation.
- Conditional branches become alternatives.
- **Computed values must be invertible.** Linear expressions (`orbit - 1200`) are inverted automatically. Others need either a user-supplied inverse, or the format is marked **render-only**, with an explanation.
- Parsing can be **ambiguous** (e.g. the era's unit written with two digits). The parser returns every valid candidate, and the UI asks the user to choose.

- **Write-only formats**: a format that can't say which date it means (it leaves out the era's unit, or a unit between two it shows: "9 Third" has no orbit) still writes dates, but is marked write-only with the reason and can't be used to read text.

## First implementation (spec 009)

- Reading gives every date the text can mean. Literal text, member names and the era's name match regardless of capital letters; a run of spaces matches any run of spaces.
- A padded field reads with or without its leading zeros, except where it touches another field with no literal between, when it reads exactly its width. Unpadded touching fields try every split.
- Readings the calendar refuses are dropped. With none left, the reason is given: where the text stops fitting and what was expected there, two values for one unit that disagree (a number and a name, or a unit above the era's unit that doesn't match the date), or the calendar's own refusal.
- A pattern that stops above the smallest unit reads partial dates; the missing units take their first values.
- Reading is built and tested in the formats component; the app first uses it when the converter takes typed dates, in a later feature.

# Round-trip test

Every format gets a **round-trip test** in the editor: render a range of dates, parse them back, and flag any mismatches. The same property is enforced in the automated test suite; see [presets and validation](/product/presets-and-validation.md).
