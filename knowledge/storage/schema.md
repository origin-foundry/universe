---
type: Data Model
title: Storage schema
description: The SQLite project file - definition tables and disposable cache tables.
tags: [storage, sqlite, schema]
status: draft
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
---

# File

One `.universe` file per project (SQLite, with `PRAGMA application_id = 0x554E4956`, "UNIV", so it's recognisable).

- **Versions**: `meta.schema_version` is the file's schema version. Each feature that needs new tables or columns adds a numbered migration. The app refuses files from a newer schema version. It migrates older ones on open, after copying the original beside it as `<name>.v<old>.universe`.
- **Working copy**: an open universe is held in an in-memory database; changes reach the file only when the owner saves (document model, spec 003).
- **Crash-safe saves**: a save writes a temporary file in the same folder and atomically renames it over the original, so the file is always the complete old or the complete new version.
- **One writer**: the app holds an exclusive lock on the file while it's open; a second open reports the file as in use.
- **Foreign keys**: switched on for every connection, so deleting a row deletes what belongs to it (`ON DELETE CASCADE`).
- **Built so far**:
  - Schema version 1 (spec 003) has `meta` and `universe(id, name)`.
  - Version 2 (spec 007) adds the calendar tables under "Calendar tables as built" below.
  - Version 3 (spec 008) adds the `anchor` table, also below.
  - Version 4 (spec 009) adds the `format` table, also below.
  - The other tables are the target design, added by the features that use them.

## Calendar tables as built (version 2)

```
calendar(id, seq, name)                          -- seq orders the universe's list; name unique
unit(id, calendar_id, seq, kind, name, first_number,
     member_unit_name, member_unit_first_number, -- groupings only
     length,                                     -- the fundamental unit only: SI seconds as the text typed, exact
     parts)                                      -- subdivisions only
unit_member(id, unit_id, seq, name, count)
rule(id, unit_id, seq, condition_src, member, change)
era(id, calendar_id, seq, name, unit_name)
```

- `unit.kind` is `grouping`, `fundamental` or `subdivision`. `seq` orders the single chain from top to bottom; `parent_id` will replace or join it when units can branch.
- A grouping's member unit is two columns on its row, not a row of its own.
- An adjustment rule is stored as its condition's source text, the member it changes and the change; `modification_json` will replace the last two when changes other than lengthening or shortening exist.
- There are no clock or pre-epoch columns yet: every calendar counts the absolute timeline.
- A calendar keeps its `id` across edits, so anchors can refer to it.

```
anchor(calendar_id,              -- the anchored calendar; one row at most per calendar
       kind,                     -- 'fixed' (fix the moment) or 'equation' (keep the equation)
       base_calendar_id,         -- the base calendar; NULL once it's deleted
       base_name,                -- the base calendar's name, written only when it's deleted
       date, base_date,          -- the anchor date and base date as entered: [412,3,9]
       at_secs, at_nanos)        -- 'fixed' only: the instant both dates started at
```

- The one calendar with no `anchor` row is the universe's **reference calendar**, at the hidden zero. At most one calendar has no row; once the reference calendar is anchored or deleted, none does.
- The epoch itself is never stored: it's worked out from the anchor whenever calendars are placed. A fixed anchor keeps `at`, the moment its own date starts at; an equation anchor works it out from the base calendar each time (see [eras and epoch](/calendar/eras-and-epoch.md)).
- `at_secs` and `at_nanos` are a value on the hidden timeline. They stay in the file: no screen or command shows them ([ADR 0002](/decisions/0002-hidden-absolute-timeline.md)).

```
format(id, calendar_id, seq,     -- seq orders the calendar's formats
       name,                     -- trimmed; unique in the calendar
       template,                 -- the pattern exactly as typed
       main)                     -- 1 on the calendar's main format, at most one per calendar
```

- A calendar with no `main = 1` row uses its generated format, which isn't stored (see [format templates](/formats/templates.md)). A file migrated from version 3 has no formats.
- `kind` and `inverse_overrides_json` of the target `format` table below come with duration formats and computed values; names per locale with locales.

# Schema

## Definition tables

```
meta(schema_version, app_version, created_at)                -- created_at: whole seconds since 1970 UTC
universe(id, name, display_calendar_id, display_format_id)    -- GCT zero is implicit, never stored as user data
system(id, name, galactic_pos_xyz, galactic_vel_xyz, sbt_rate_override, sbt_offset)
body(id, system_id, parent_id, name, kind, mass, radius, j2)
body_elements(body_id, a, e, i, raan, argp, mean_anomaly, epoch)  -- or state vectors
spin_model(body_id, period, period_drift, obliquity, pole_ra, pole_dec, precession_rate, tidally_locked)
observer(id, kind, body_id, lat, lon, alt, name)
ship_segment(id, observer_id, seq, kind, params_json)
calendar(id, name, clock_observer_id, clock_kind, pre_epoch_mode)   -- anchors: the anchor table above
zone_scheme(id, body_id, kind, prime_meridian_lon)
zone(id, scheme_id, name, abbr, base_offset, lon_min, lon_max)
zone_transition(id, zone_id, seq, kind, effective_from, offset, recurrence_src, recurrence_ast)
observer_zone(observer_id, zone_id)
unit(id, calendar_id, parent_id, name, plural, kind, params_json)
unit_member(id, unit_id, seq, name, length, intercalary)
cycle(id, calendar_id, over_unit_id, period, anchor, skip_expr)
cycle_member(cycle_id, seq, name)
rule(id, unit_id, seq, predicate_src, predicate_ast, modification_json)
function_def(id, calendar_id, name, params, body_src, body_ast)
era(id, calendar_id, seq, name, abbr, start, direction, has_zero)
locale(id, calendar_id, code, name)
name_set(locale_id, target_kind, target_id, full, abbr)
format(id, calendar_id, name, kind, template, inverse_overrides_json)
event(id, name, gct_instant, end_instant, notes, tags)
```

Expressions are stored as **source text** (what the user wrote), re-checked when the file is opened. **AST JSON** (validated, versioned) will be stored alongside it once something needs to read an expression without parsing it. See the [expression language](/language/expression-language.md).

## Cache tables (disposable)

```
ephemeris_checkpoint(system_id, config_hash, t, state_blob)
ephemeris_dense(system_id, config_hash, t0, t1, coeff_blob)
event_cache(system_id, config_hash, event_kind, params_hash, t)
clock_table(observer_id, config_hash, t_sbt, tau)
calendar_table(calendar_id, config_hash, unit_id, index, start_fixed)
```

Caches are keyed by a hash of their inputs, so stale entries are never served. A **"Compact project"** command drops all caches to make files small for sharing.

Related: [integration](/physics/integration.md), [conversion](/calendar/conversion.md), [JSON export](/storage/export.md).
