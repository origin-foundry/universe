---
type: Data Model
title: Storage schema
description: The SQLite project file - definition tables and disposable cache tables.
tags: [storage, sqlite, schema]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:12:40Z }
---

# File

One `.universe` file per project (SQLite, with `PRAGMA application_id` set so it's recognisable).

# Schema

## Definition tables

```
meta(schema_version, created_at, app_version)
universe(id, name, display_calendar_id, display_format_id)    -- GCT zero is implicit, never stored as user data
system(id, name, galactic_pos_xyz, galactic_vel_xyz, sbt_rate_override, sbt_offset)
body(id, system_id, parent_id, name, kind, mass, radius, j2)
body_elements(body_id, a, e, i, raan, argp, mean_anomaly, epoch)  -- or state vectors
spin_model(body_id, period, period_drift, obliquity, pole_ra, pole_dec, precession_rate, tidally_locked)
observer(id, kind, body_id, lat, lon, alt, name)
ship_segment(id, observer_id, seq, kind, params_json)
calendar(id, name, clock_observer_id, clock_kind, epoch_anchor_json, pre_epoch_mode)
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
era(id, calendar_id, seq, name, abbr, start, direction, year_zero)
locale(id, calendar_id, code, name)
name_set(locale_id, target_kind, target_id, full, abbr)
format(id, calendar_id, name, kind, template, inverse_overrides_json)
event(id, name, gct_instant, end_instant, notes, tags)
```

Expressions are stored as both **source text** (what the user wrote) and **AST JSON** (validated, versioned). See the [expression language](/language/expression-language.md).

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
