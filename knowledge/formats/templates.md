---
type: Design
title: Format templates
description: Template syntax, locales, built-in formats and duration formats.
tags: [formats, templates, locales]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:54Z }
---

# Syntax

```
{weekday:name}, {day:ord} {month:name} {year:era}{?era.before} {era:abbr}{/}
```

- **Field tokens:** any unit, cycle member or era field, with modifiers: `:name`, `:abbr`, `:ord` (ordinal), `:04` (zero-padded), `:roman`, `:words`
- **Zone tokens:** `{zone:name}`, `{zone:abbr}`, `{zone:offset}`; see [time zones](/calendar/time-zones.md)
- **Conditionals:** `{?condition}…{:else}…{/}`
- **Computed values:** `{= expression}` using the [expression language](/language/expression-language.md)
- **Literals and escapes**

# Locales

Each calendar can define several name sets (languages, dialects, formal vs. colloquial). A format chooses a locale or follows the universe default.

# Built-in formats

Each calendar gets three generated formats automatically: **canonical** (unambiguous, machine-friendly, sortable), **long** and **short**. Users can edit or replace them.

# Duration formats

The same template machinery formats durations ("3 orbits, 12 turns"), using the display calendar's units by default. See [display units](/time/display-units.md).

# Parsing

Every format is also a parser; see [format parsing](/formats/parsing.md).
