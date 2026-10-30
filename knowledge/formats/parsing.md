---
type: Design
title: Format parsing
description: Compiling templates into parsers, invertibility and ambiguity.
tags: [formats, parsing]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:54Z }
---

# Rules

Each [format template](/formats/templates.md) compiles into a parser as well as a renderer.

- Field tokens and named members parse naturally. Names match case-insensitively, by full name or abbreviation.
- Conditional branches become alternatives.
- **Computed values must be invertible.** Linear expressions (`year - 1200`) are inverted automatically. Others need either a user-supplied inverse, or the format is marked **render-only**, with an explanation.
- Parsing can be **ambiguous** (e.g. a two-digit year). The parser returns every valid candidate, and the UI asks the user to choose.

# Round-trip test

Every format gets a **round-trip test** in the editor: render a range of dates, parse them back, and flag any mismatches. The same property is enforced in the automated test suite; see [presets and validation](/product/presets-and-validation.md).
