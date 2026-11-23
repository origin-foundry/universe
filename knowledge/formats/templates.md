---
type: Design
title: Format templates
description: Template syntax, locales, built-in formats and duration formats.
tags: [formats, templates, locales]
status: draft
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
---

# Syntax

```
{rota:name}, {sol:ord} {span:name} {orbit:era}{?era.before} {era:abbr}{/}
```

On Earth the same template reads `{weekday:name}, {day:ord} {month:name} {year:era}…`.

- **Field tokens:** any unit, cycle member or era field, with modifiers: `:name`, `:abbr`, `:ord` (ordinal), `:04` (zero-padded), `:roman`, `:words`
- **Zone tokens:** `{zone:name}`, `{zone:abbr}`, `{zone:offset}`; see [time zones](/calendar/time-zones.md)
- **Conditionals:** `{?condition}…{:else}…{/}`
- **Computed values:** `{= expression}` using the [expression language](/language/expression-language.md)
- **Literals and escapes**

## First implementation (spec 009)

The first formats build a subset of this syntax, so later features add to it without changing what a stored pattern means:

- `{unit}` writes a unit's number; `{unit:02}` pads it with zeros to a width of 1 to 18 digits, after any minus sign; `{unit:name}` writes a member unit's member name, or the number for a member with none; `{era}` writes the era's name. `{{` and `}}` write literal braces.
- Fields can name the era's unit, any unit below it, and units above it (worked out from the date), in any order and any number of times.
- A pattern is checked against its calendar: unknown units or modifiers, `:name` where no member has a name, unclosed or stray braces, and a pattern with no field are refused with the position and reason.
- Each calendar has named formats and one **main format**, used wherever the app writes its dates (the converter's results, the anchor's read-back sentence). With none chosen, the main format is the **generated format**: each unit's name and number from the era's unit down, with member names in parentheses where every member has one ("orbit 412, span 3 (Third), sol 9, bell 0, pulse 0"). It follows the definition and isn't stored. It stands in for the three built-in formats below until they come.
- A format whose fields don't run from the era's unit down without a gap is **write-only**; see [format parsing](/formats/parsing.md).

# Locales

Each calendar can define several name sets (languages, dialects, formal vs. colloquial). A format chooses a locale or follows the universe default.

# Built-in formats

Each calendar gets three generated formats automatically: **canonical** (unambiguous, machine-friendly, sortable), **long** and **short**. Users can edit or replace them.

# Duration formats

The same template machinery formats durations ("3 orbits, 12 turns"), using the display calendar's units by default. See [display units](/time/display-units.md).

# Parsing

Every format is also a parser; see [format parsing](/formats/parsing.md).
