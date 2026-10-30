---
type: Design
title: Observational calendars
description: Units whose boundaries are set by sky conditions at a site.
tags: [calendar, observational, sky]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:11:22Z }
---

# Definition

A unit's boundaries can be determined by a sky condition at a site, evaluated once per fundamental unit. For example: "a month begins on the evening when, at sunset at site X, the moon's elongation from the star exceeds 10° and its altitude exceeds 5°".

The condition is written with the sky functions of the [expression language](/language/expression-language.md). The engine evaluates it through the [event finder](/physics/event-finder.md) and caches the results; see the cumulative-table strategy in [conversion](/calendar/conversion.md).

Observational units work before the epoch as far back as the ephemeris is computed; see [eras and epoch](/calendar/eras-and-epoch.md).

# Validation

The observational Islamic preset is compared against published crescent-visibility predictions (approximately). See [presets and validation](/product/presets-and-validation.md).
