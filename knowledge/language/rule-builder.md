---
type: Design
title: Rule builder
description: The visual builder that produces the same AST as the expression language.
tags: [language, ui, rules]
status: draft
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
---

# Patterns

Forms and dropdowns cover the common patterns:

- "every *N* *units*, except every *M*, except every *K*"
- "in *units* numbered {list} within a cycle of *N*" (e.g. the tabular Islamic leap years)
- "when *sky condition* at *site*"

# Round-tripping

The builder produces the **same AST** as the [expression language](/language/expression-language.md). Toggling to **Advanced** shows the generated expression for editing. Edited expressions that still match a builder pattern can be switched back.

Used for [adjustment rules](/calendar/adjustment-rules.md) and [time zone](/calendar/time-zones.md) transitions.
