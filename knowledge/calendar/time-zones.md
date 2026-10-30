---
type: Design
title: Time zones
description: Per-body local time schemes, from planetary standard to full zone rules.
tags: [calendar, time-zones, local-time]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:22Z }
---

# Schemes

Each [body](/physics/bodies.md) has a **time zone scheme**. Different bodies can use different schemes in the same universe. See [ADR 0003](/decisions/0003-per-body-time-zones.md).

| Scheme | Description |
|---|---|
| **Planetary standard** | One time for the whole body, referenced to a prime meridian (like UTC). |
| **Per-site solar** | Each site's local time follows its own longitude: mean solar time, or apparent solar time (sundial time, varying through the year). |
| **Named zones** | Zones with fixed offsets from the planetary standard, assigned to sites or longitude bands. |
| **Zone rules** | Named zones whose offsets change over time: historical changes and recurring seasonal shifts (like daylight saving). |

Schemes can be mixed on one body (e.g. a planetary standard for official use plus per-site solar time for remote outposts).

# Zone rules

A time-ordered list of **transitions** per zone, each one either:

- a one-off change ("from year 412 of the Second Era, offset becomes +3 tenths"), or
- a recurring rule ("from the 2nd Restday after the spring equinox until the autumn equinox, add 1 tenth"), written with the [rule builder](/language/rule-builder.md) or [expression language](/language/expression-language.md), and able to reference sky events.

# Behaviour

- A calendar date can carry a zone; the converter shows local times wherever relevant.
- Ambiguous or skipped local times at transitions are reported explicitly, never resolved silently.
- Formats get `{zone:name}`, `{zone:abbr}` and `{zone:offset}` tokens, and the parser accepts them. See [format templates](/formats/templates.md).

# Storage

See the `zone_scheme`, `zone`, `zone_transition` and `observer_zone` tables in the [storage schema](/storage/schema.md).
