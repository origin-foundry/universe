---
type: Design
title: JSON export
description: Diff-friendly export and import of definitions, without caches.
tags: [storage, export, json]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:12:40Z }
---

# Purpose

The primary format is SQLite (see [storage schema](/storage/schema.md)), which isn't diff-friendly. A **JSON export/import** of the definition tables (no caches) supports diffing, version control and sharing presets.

Scheduled for phase 6 in the [delivery phases](/product/delivery-phases.md). Mitigates risk 5 in [risks](/risks.md).
