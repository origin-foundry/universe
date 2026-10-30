# Universe — Design Document

> **Superseded.** This document has been split into the OKF knowledge bundle in [`knowledge/`](knowledge/index.md), which is now the source of truth. This file is kept only for reference and should not be edited.

> Working title. A desktop app for defining physically grounded, non-Earth calendars across multiple star systems, and converting dates between them and between observers whose clocks run at different rates.

Status: **Draft 2** — open questions resolved (see §15). For review before any code is written.

---

## 1. Goals and non-goals

### Goals

- Let a user build a **universe**: several star systems, each with stars, planets and moons defined by masses and orbital elements.
- Simulate that universe with **N-body physics** over roughly ±100,000 years with second-level timing precision.
- Derive natural **cycles** (days, years, lunations, seasons, eclipses) from the simulation instead of typing them in.
- Let the user define **calendars** with no built-in notion of day/week/month/year: arbitrary nested units, parallel cycles, variable-length units, named members, intercalary units, eras and leap rules.
- Support three kinds of leap logic: **hand-authored**, **auto-suggested** from simulated periods, and **observational** (driven by sky events).
- Define **date formats** with conditional logic that can both **render** and **parse** dates.
- Model **proper time**: observers on planets, in orbit, or on ships flying full trajectories experience time at different rates, and each calendar is tied to a specific clock.
- Ship real-world presets (Solar System; Gregorian, Julian, Islamic, Hebrew calendars) that double as a correctness test suite.

### Non-goals (for now)

- Full general relativity. We use weak-field post-Newtonian corrections in-system and special relativity between systems (see §4.4), which is accurate well beyond what fiction needs.
- Rigid-body spin dynamics. Rotation is modelled kinematically with secular drift terms, not integrated (see §5.3).
- Stellar evolution, geologic or cosmological timescales.
- Collaboration or cloud sync. Projects are local files.

---

## 2. Key concepts

| Term | Meaning |
|---|---|
| **Universe** | One project. Contains systems, observers, calendars, formats and events. |
| **Galactic frame** | The single inertial reference frame the whole universe is described in. Its coordinate time is the absolute timeline. |
| **GCT** | *Galactic Coordinate Time*. The absolute timeline, counted in SI seconds from a hidden technical zero. Every instant in the app is ultimately a GCT value, but **users never see it**: it exists only so points on different calendars can be equated. |
| **System** | A star system: a set of gravitationally bound bodies with a position and velocity in the galactic frame. |
| **SBT** | *System Barycentric Time*. Coordinate time at a system's barycentre, related to GCT by a fixed rate and offset. |
| **Body** | A star, planet, moon or other massive object inside a system. |
| **Observer** | Something that experiences time: a site on a body's surface, an orbiting station, or a ship. |
| **Clock** | The proper time τ experienced by an observer, as a function of GCT. |
| **Cycle** | A recurring physical phenomenon detected from the simulation (e.g. "local noon at site X", "new moon of body Y"). |
| **Calendar** | A naming scheme for instants, bound to one clock, made of units, cycles, rules and eras. Its epoch is the start of its first era. |
| **Time zone scheme** | How local time is reckoned on a body: planetary standard, per-site solar time, named zones, or zones with historical and seasonal rules (§6.10). |
| **Unit** | A named division of time inside a calendar (what Earth calendars call day, month, year…). |
| **Format** | A template that renders a calendar date as text and parses text back to a date. |
| **Event** | A named instant (or interval) on the absolute timeline, shown in every calendar. |

### The two questions a date answers

A date can mean two different things, and the app always keeps them apart:

1. **Where is this on the absolute timeline?** (a GCT instant — used for "what was happening elsewhere at the same moment")
2. **How much time did this observer live through?** (a proper-time value on that observer's clock)

A traveller's ship calendar and the home planet's calendar can name the same GCT instant as dates years apart, and both are "correct". The converter shows both.

> **Simultaneity is a convention.** "The same moment on two planets" is defined as "the same GCT". This is a choice of frame, which the app states openly. Light-delay queries (§8.6) answer the separate question "when could B have *learned* about A?"

---

## 3. Units and display

**SI internally, renamed in the UI.** All physics and storage use SI units (seconds, metres, kilograms). The UI never shows Earth units or raw GCT values unless the user asks:

- Each universe has a **display calendar** (user-chosen) used to show durations and dates by default.
- Durations are shown in that calendar's units ("3 cycles, 14 tenths"), with an optional "show SI" toggle for debugging and for the Earth presets.
- Physical inputs (mass, distance) can be entered in user-defined units (e.g. "1 home-mass = 5.2 × 10²⁴ kg") or in SI.

---

## 4. Time model

### 4.1 Representation

| Domain | Type | Why |
|---|---|---|
| Absolute instants (GCT, SBT, τ) | `Instant { secs: i64, nanos: u32 }` | Exact, ordered, ±292 billion years range, nanosecond resolution. |
| Calendar arithmetic | Exact rationals (`i128 / i128`) and integers | Leap rules and unit lengths must never accumulate float error. |
| Physics integration | `f64` seconds relative to a simulation epoch near the middle of the range | At ±3.2 × 10¹² s (100k years), f64 resolution is ~0.5 ms, comfortably within the second-precision target. |

Conversions between `Instant` and physics `f64` happen at one well-tested boundary.

GCT zero is a **hidden technical zero**, chosen by the app when a universe is created (in practice the physics epoch of the first system). It has no name and no meaning to the user; every date shown is expressed in some calendar. A "Show GCT" option exists only in a developer/debug panel.

### 4.2 Frame chain

```
GCT  ──(linear: rate + offset per system)──▶  SBT_system
SBT  ──(integrated: dτ/dt along worldline)──▶  τ_observer
```

- **GCT → SBT** is linear. The rate comes from the system's velocity in the galactic frame (Lorentz factor) and an optional constant gravitational potential term. Users can override it.
- **SBT → τ** is integrated along the observer's path through the system's simulated gravitational field (§4.3). The result is cached as a monotonic table, so the inverse (τ → SBT) is a binary search plus interpolation.

### 4.3 Proper time in-system (weak field)

For an observer at position **x** with velocity **v** in the system barycentric frame:

```
dτ/dt ≈ 1 + Φ(x)/c² − v²/(2c²)
```

where Φ is the summed Newtonian potential of all bodies (negative). This is the same first-order approximation used for real-world TT/TCB and GPS clock corrections, and is accurate to roughly 10⁻¹⁶ per unit time for planetary conditions.

When a ship's speed in-system exceeds ~0.01c, the app switches to the exact special-relativistic factor `sqrt(1 − v²/c²)` combined with the potential term, and warns that Newtonian trajectory dynamics are becoming approximate.

### 4.4 Ships between systems

You asked for full trajectories. The plan is a **hybrid** model, honest about where each piece of physics is valid:

| Segment | Dynamics | Clock |
|---|---|---|
| **In-system** (departure, arrival, manoeuvres) | Test particle integrated inside the system's N-body simulation, with a thrust schedule (burn start, duration, direction, acceleration). | Weak-field formula (§4.3). |
| **Interstellar** | Special-relativistic motion in the galactic frame: constant proper acceleration ("hyperbolic motion"), coast and deceleration phases, solved in closed form. | Exact: e.g. `t = (c/a)·sinh(aτ/c)` during constant acceleration. |
| **Handoff** | At a configurable distance from a system's barycentre, state vectors are converted between the system frame and the galactic frame (Lorentz boost by the system's velocity). | Continuous across the boundary. |

Fully self-consistent relativistic N-body dynamics is out of scope. For any speed and acceleration a fictional ship will plausibly use, the hybrid model agrees with it to well below a second over a voyage.

---

## 5. Physics

### 5.1 Defining bodies

Each body has:

- mass, radius, optional oblateness (J2, for precession effects)
- a parent body (star → planet → moon), forming a hierarchy
- **osculating orbital elements** relative to its parent at the system epoch: semi-major axis *a*, eccentricity *e*, inclination *i*, longitude of ascending node Ω, argument of periapsis ω, mean anomaly *M*
- a spin model (§5.3)

Elements are converted to Cartesian state vectors in **Jacobi coordinates**, so hierarchical systems (binary stars, moons of moons) start in sensible configurations. Advanced users can enter state vectors directly.

Systems themselves have a galactic position and velocity (straight-line motion; galactic orbit ignored over ±100k years).

### 5.2 Integration

- **Integrator:** **REBOUND** (C library), bound into Rust via FFI (`bindgen` + a thin safe wrapper crate, `rebound-sys` / `uni-physics`). IAS15 (adaptive 15th-order Gauss–Radau) is the default for accuracy, with WHFast (Wisdom–Holman symplectic) as a fast option for long, well-behaved runs. REBOUND is vendored and compiled by `build.rs` with the `cc` crate, so no separate install is needed. It's GPL-3.0, which is fine for personal use (§15). The rest of the app talks to it through an `Integrator` trait, so it stays replaceable.
- **Ship thrust:** REBOUND's `additional_forces` callback is used to apply ship thrust schedules to test particles.
- **Ephemeris cache:** the simulation is deterministic given its inputs. Results are stored as **checkpoints** (full state every N steps) plus **dense output** (Hermite-interpolable state at a finer interval). Queries interpolate; queries outside the cached span extend the integration forward or backward from the nearest checkpoint.
- **Cache key:** a hash of every input that affects physics. Editing a body invalidates only that system's cache.
- **Chaos:** N-body systems are chaotic over long spans. The app will show a **reliability indicator** (from comparing two runs with tiny perturbations) so users know when far-future dates stop being meaningful.

### 5.3 Rotation

Spin isn't produced by translational N-body integration, so each body has a kinematic spin model:

- sidereal rotation period at epoch, plus a **secular drift** term (e.g. tidal braking, in seconds of period per century)
- axial tilt (obliquity) and pole direction
- axial **precession** rate and optional nutation terms
- option to **tidally lock** to the parent (rotation derived from the orbit)

### 5.4 Event finder

Derived cycles come from root-finding on the ephemeris (bracketing plus Brent's method). Built-in event types:

| Event | Definition |
|---|---|
| Transit / local noon & midnight | A star crosses the meridian of a surface site. |
| Rise / set | A body's centre or limb crosses the horizon (with optional refraction). |
| Equinox / solstice | The star's declination as seen from a planet crosses zero or reaches an extremum. |
| Periapsis / apoapsis | A body's distance to its parent reaches a minimum or maximum. |
| Conjunction / opposition / phase | Elongation between two bodies as seen from a third reaches a given value (e.g. new moon). |
| Eclipse | Shadow geometry overlap. |
| Custom | Any expression over ephemeris quantities crossing a threshold (§7). |

**Mean periods** are fitted over a user-chosen window (e.g. "mean tropical year over ±1,000 years"), because N-body periods aren't constant. Both the event list and the mean value are available to calendars.

---

## 6. Calendar engine

### 6.1 Overview

A calendar has:

- a **clock** it counts (an observer's τ, a system's SBT, or GCT)
- an **epoch**: the start of its **first defined era** (§6.8). Time before the epoch is fully supported (§6.9).
- a **unit graph**: the units and how they relate
- **parallel cycles**: independent counters such as weeks
- **rules**: leap and adjustment logic
- **eras**: how the top-level count is numbered and labelled
- **formats** (§8)

### 6.2 The fundamental unit

Every calendar has exactly one **fundamental unit**: the smallest unit that's counted as a whole integer (on Earth, the day). All other units are either subdivisions of it or groupings of it. The fundamental unit is either:

- **Fixed:** a constant duration on the calendar's clock (e.g. 86,400 s, or "the mean solar day fitted from simulation, frozen at epoch"). Fast and exact.
- **Event-delimited:** each instance runs from one physical event to the next (e.g. sunrise to sunrise at site X). Lengths vary naturally.

Each instance gets an integer **fixed number** counted from epoch (fixed number 0 is the first fundamental unit of the first era; earlier units are negative). It's the calendar's equivalent of the Julian Day Number, and the backbone every other unit is built on.

### 6.3 Unit kinds

| Kind | Description | Earth example |
|---|---|---|
| **Subdivision** | Parent split into *n* equal children. Can apply to event-delimited parents, giving variable-length "temporal hours". | day → 24 hours → 60 minutes |
| **Grouping** | A sequence of child counts per position, with optional **named members**. Counts can be a fixed list, a list adjusted by rules, or a **length expression**. | year → 12 months [31, 28, 31, …] |
| **Event-delimited grouping** | Boundaries come from physical events, snapped to the fundamental unit. | Observational lunar month |
| **Intercalary member** | A member that belongs to its parent but sits outside the regular sequence (and can be excluded from parallel cycles). | Epagomenal days; the World Calendar's "Worldsday" |

Units form a **tree** under the top-level unit. Parallel cycles hang off the tree without nesting.

### 6.4 Parallel cycles

A parallel cycle is a modular counter over some unit, with:

- period *n* and optional named members
- an **anchor**: which member falls on a given fixed number
- an optional **skip rule**: units the cycle doesn't advance on (for intercalary days)

Multiple parallel cycles can coexist, which covers the 7-day week, market weeks, and Maya-style interlocking counts (Tzolk'in 13 × 20 against Haab' 365).

### 6.5 Leap rules

A rule is **predicate → modification**, attached to a unit:

- **Predicate:** an expression over the unit's index and context (e.g. `year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)`), or an event query for observational calendars.
- **Modification:** extend or shorten a named member by *k* children; insert or remove a member at a position; insert an intercalary member.

For complex calendars (the Hebrew calendar is the reference stress test), a unit can instead define:

- a **length expression**: `length(year) = …`
- optionally a **start expression**: `start(year) = …` giving the fixed number of the unit's first day in closed form

These use the expression language's pure functions (§7), so molad and postponement logic can be written directly.

### 6.6 Auto-suggested rules

Given a target ratio from the simulation (e.g. mean tropical year ÷ mean solar day = 412.3719…), the suggester:

1. Computes **continued-fraction convergents** (e.g. 412/1, 4123/10, 21031/51 …) giving candidate cycle lengths.
2. Converts promising candidates into **nested rule sets** ("leap every 3rd, except every 51st, …") by greedy decomposition.
3. Scores each option on **drift per millennium** against both the mean period and the actual simulated event series, plus **complexity** (number of rule clauses).
4. Shows the results as a Pareto chart (accuracy vs. simplicity). The user picks one, then edits it like any hand-authored rule.

### 6.7 Observational calendars

A unit's boundaries can be determined by a sky condition at a site, evaluated once per fundamental unit. For example, "a month begins on the evening when, at sunset at site X, the moon's elongation from the star exceeds 10° and its altitude exceeds 5°". The engine evaluates this through the event finder and caches the results.

### 6.8 Eras

A calendar has an ordered list of **one or more eras**. Eras map the top-level unit's index to displayed numbers and labels:

- start (by fixed number, or by a timeline event, for regnal eras)
- numbering direction (counting up or counting down to the era boundary)
- whether a year zero exists
- label and abbreviation per locale

**The first era defines the calendar's epoch.** You place it on the hidden timeline by equating one of its dates with a date in another calendar, with a timeline event, or with a sky event (e.g. "year 1 began at the first equinox after the Landing"). Later eras start relative to the calendar's own dates.

### 6.9 Time before the epoch

Dates before the first era are always valid; the calendar's rules simply run backwards (**proleptic** extension). How they're labelled is configurable per calendar:

- a **pre-epoch era** (e.g. "Before Founding", counting down, with or without a year zero)
- **astronomical numbering** (year 0, −1, −2, …)
- both, with formats choosing which to show

Leap rules, parallel cycles and observational units all apply before the epoch exactly as after it (observational ones as far back as the ephemeris is computed).

### 6.10 Local time and time zones

Time-of-day units on a body's surface depend on where you are. Each body has a **time zone scheme**, and different bodies can use different schemes in the same universe:

| Scheme | Description |
|---|---|
| **Planetary standard** | One time for the whole body, referenced to a prime meridian (like UTC). |
| **Per-site solar** | Each site's local time follows its own longitude: mean solar time, or apparent solar time (sundial time, varying through the year). |
| **Named zones** | Zones with fixed offsets from the planetary standard, assigned to sites or longitude bands. |
| **Zone rules** | Named zones whose offsets change over time: historical changes and recurring seasonal shifts (like daylight saving). |

Zone rules are a time-ordered list of **transitions** per zone, each one either:

- a one-off change ("from year 412 of the Second Era, offset becomes +3 tenths"), or
- a recurring rule ("from the 2nd Restday after the spring equinox until the autumn equinox, add 1 tenth"), written with the rule builder or expression language and able to reference sky events.

Schemes can be mixed on one body (e.g. a planetary standard for official use plus per-site solar time for remote outposts). A calendar date can carry a zone, and the converter shows local times wherever relevant. Ambiguous or skipped local times at transitions are reported explicitly, never resolved silently. Formats get `{zone:name}`, `{zone:abbr}` and `{zone:offset}` tokens, and the parser accepts them.

### 6.11 Conversion algorithm

Converting an instant to a date:

1. Map the instant to the calendar's clock (§4.2).
2. Find the **fixed number** (fixed fundamental unit: division; event-delimited: binary search in the cached event table).
3. Walk the unit tree top-down to resolve each field.

Step 3 uses the fastest strategy available for each unit:

| Strategy | When it applies | Cost |
|---|---|---|
| **Supercycle** | The rules are periodic (e.g. Gregorian repeats every 400 years / 146,097 days). The engine detects the period and precomputes one table. | O(1) |
| **Start expression** | The user supplied `start(unit)` in closed form. | O(log n) search over the index |
| **Cumulative table** | Everything else (observational, irregular). Lazily built, checkpointed, cached in the project file. | O(log n) lookup |

The reverse (date → instant) uses the same machinery. Invalid dates (e.g. the 30th of a 29-day month) are reported with the reason, never silently wrapped.

### 6.12 Example: Gregorian, expressed in the model

```yaml
calendar: Gregorian
clock: observer "Earth / Greenwich"   # τ on Earth's surface ≈ TT
epoch: start of first era (AD 1, January 1)
zones: Earth uses "zone rules" (IANA-style)
fundamental:
  unit: day
  kind: fixed
  duration: 86400 s
units:
  - year:
      kind: grouping
      children: month
      members: [January: 31, February: 28, March: 31, April: 30, May: 31, June: 30,
                July: 31, August: 31, September: 30, October: 31, November: 30, December: 31]
      rules:
        - when: "year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)"
          do: extend February by 1
  - day:
      kind: subdivision
      children: [hour × 24, minute × 60, second × 60]
cycles:
  - week: { period: 7, of: day, members: [Monday … Sunday], anchor: "fixed 1 = Monday" }
eras:
  - AD: { from: year 1, direction: up }          # first era → defines the epoch
pre_epoch:
  - BC: { direction: down, year_zero: false }
```

(This shows the model, not the storage format. Calendars live in SQLite, §9.)

---

## 7. Expression language and rule builder

### 7.1 Language

A small, typed, side-effect-free expression language, compiled to an AST and evaluated in Rust.

- **Types:** integer, rational, boolean, angle, duration, instant, string
- **Operators:** arithmetic, `%` (floored modulo), comparisons, `&&`, `||`, `!`, `if … then … else`
- **Bindings:** `let x = … in …`
- **Pure user-defined functions**, so complex calendars can be built from reusable pieces. Recursion is allowed only with a statically bounded depth, so evaluation always terminates.
- **Context variables:** the current unit indices (`year`, `month`, … using the user's own unit names), era fields, the fixed number
- **Sky functions** (observational rules only): `elongation(a, b, from)`, `altitude(body, site)`, `phase(body)`, `distance(a, b)`, `event_between(kind, t0, t1)`

Every expression is type-checked when it's saved, with errors pointing at the exact position.

### 7.2 Visual builder

Forms and dropdowns cover the common patterns:

- "every *N* *units*, except every *M*, except every *K*"
- "in *units* numbered {list} within a cycle of *N*" (e.g. the tabular Islamic leap years)
- "when *sky condition* at *site*"

The builder produces the **same AST** as the text language. Toggling to **Advanced** shows the generated expression for editing. Edited expressions that still match a builder pattern can be switched back.

---

## 8. Formats

### 8.1 Templates

```
{weekday:name}, {day:ord} {month:name} {year:era}{?era.before} {era:abbr}{/}
```

- **Field tokens:** any unit, cycle member or era field, with modifiers: `:name`, `:abbr`, `:ord` (ordinal), `:04` (zero-padded), `:roman`, `:words`
- **Conditionals:** `{?condition}…{:else}…{/}`
- **Computed values:** `{= expression}` using the expression language
- **Literals and escapes**

### 8.2 Locales

Each calendar can define several name sets (languages, dialects, formal vs. colloquial). A format chooses a locale or follows the universe default.

### 8.3 Parsing

Each format compiles into a parser as well as a renderer.

- Field tokens and named members parse naturally. Names match case-insensitively, by full name or abbreviation.
- Conditional branches become alternatives.
- **Computed values must be invertible.** Linear expressions (`year - 1200`) are inverted automatically. Others need either a user-supplied inverse or the format is marked **render-only**, with an explanation.
- Parsing can be **ambiguous** (e.g. a two-digit year). The parser returns every valid candidate, and the UI asks the user to choose.
- Every format gets a **round-trip test** in the editor: render a range of dates, parse them back, and flag any mismatches.

### 8.4 Built-in formats

Each calendar gets three generated formats automatically: **canonical** (unambiguous, machine-friendly, sortable), **long** and **short**. Users can edit or replace them.

### 8.5 Duration formats

The same template machinery formats durations ("3 orbits, 12 turns"), using the display calendar's units by default.

### 8.6 Light-delay queries (optional, phase 5)

"When could observer B first have observed event A?" Solved as the intersection of A's future light cone with B's worldline. This is useful for sci-fi plotting ("news of the coronation reaches the colony on…").

---

## 9. Storage (SQLite project file)

One `.universe` file per project (SQLite, with `PRAGMA application_id` set so it's recognisable).

### 9.1 Definition tables

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

Expressions are stored as both **source text** (what the user wrote) and **AST JSON** (validated, versioned).

### 9.2 Cache tables (disposable)

```
ephemeris_checkpoint(system_id, config_hash, t, state_blob)
ephemeris_dense(system_id, config_hash, t0, t1, coeff_blob)
event_cache(system_id, config_hash, event_kind, params_hash, t)
clock_table(observer_id, config_hash, t_sbt, tau)
calendar_table(calendar_id, config_hash, unit_id, index, start_fixed)
```

Caches are keyed by a hash of their inputs, so stale entries are never served. A **"Compact project"** command drops all caches to make files small for sharing.

### 9.3 Export

Even though the primary format is SQLite, a **JSON export/import** of definition tables (no caches) is planned, for diffing, version control and sharing presets.

---

## 10. Architecture

### 10.1 Stack

**Tauri 2** desktop app:

- **Rust core**, for numerically heavy physics and exact calendar arithmetic, with predictable performance.
- **TypeScript + React UI.** **react-three-fiber** (Three.js) for the orrery, **visx/d3** for timelines and drift charts.
- **rusqlite** for storage.

### 10.2 Rust workspace

```
crates/
  uni-time/       Instant, rationals, frame chain (GCT/SBT/τ), clock tables
  rebound-sys/    vendored REBOUND C source, build.rs, bindgen FFI
  uni-physics/    bodies, element conversion, safe REBOUND wrapper (Integrator trait), spin, event finder, ephemeris cache
  uni-zones/      time zone schemes, transitions, local-time resolution
  uni-expr/       lexer, parser, type checker, evaluator for the expression language
  uni-calendar/   unit graph, rules, cycles, eras, conversion strategies, rule suggester
  uni-format/     template compiler, renderer, parser, locales
  uni-store/      SQLite schema, migrations, repositories, cache management
  uni-presets/    Solar System + Earth calendars
  uni-app/        Tauri commands, background job runner, progress events
ui/               React app
```

Dependencies only point downward (`uni-calendar` uses `uni-expr` and `uni-time`; `uni-physics` never depends on calendars). The core crates have no Tauri dependency, so they can later be published as a library or CLI.

### 10.3 Background work

Long jobs (integrating an ephemeris, building event or calendar tables, rule suggestion) run on a Rust worker pool. They report progress to the UI, can be cancelled, and write results to the cache tables. The UI never blocks; views show "computing…" states for data that isn't ready.

---

## 11. User interface

| View | Purpose |
|---|---|
| **Universe editor** | Systems, bodies (hierarchical tree), orbital elements, spin models, galactic placement. A live mini-orrery previews edits. |
| **Time zones** | Per body: pick a scheme, draw zones on a longitude map, edit transitions and seasonal rules, preview a zone's offset over time. |
| **Observers & ships** | Surface sites (map picker on the body), stations, ships with a segment-by-segment trajectory editor and a plot of the onboard clock vs. GCT. |
| **Calendar editor** | Unit tree, members, parallel cycles, rules (builder + advanced), eras and pre-epoch labelling, epoch anchoring, locales. Live preview: a calendar grid plus a list of drift statistics. |
| **Rule suggester** | Pick a target cycle; view candidate rules on an accuracy-vs-simplicity chart; apply one. |
| **Format editor** | Template with syntax highlighting, live render, parse test box and round-trip report. |
| **Converter** | Enter an instant or a date in any calendar/format. See it in every calendar and every observer's clock side by side, with GCT and proper-time differences called out. |
| **Calendar grid** | Page-style view of any unit (e.g. a "month" equivalent) with festivals, moon phases, seasons and eclipses marked. |
| **Orrery** | 2D and 3D views of a system, scrubbable through time, with body trails and ship trajectories. Clicking a moment shows it in the converter. |
| **Timeline** | Events across the absolute timeline in parallel lanes per calendar. Includes **drift charts**: how far a calendar's units wander from the sky events they track. |

---

## 12. Presets and validation

Real-world presets serve as worked examples and as the test oracle.

| Preset | Validates |
|---|---|
| **Solar System** (Sun, eight planets, major moons; J2000 mean elements) | Integrator accuracy: positions against published ephemerides within stated tolerances over ±a few thousand years; derived tropical year ≈ 365.2422 days; synodic month ≈ 29.5306 days. |
| **Gregorian & Julian** | Arithmetic grouping rules, supercycle detection, eras without year zero, weeks. |
| **Islamic (tabular)** | Rules on a position within a 30-year cycle. |
| **Islamic (observational)** | Observational months from crescent visibility, compared against published visibility predictions (approximate). |
| **Hebrew** | Length/start expressions, molad arithmetic, postponement rules, leap months. |
| **Proper time** | GPS-orbit clock rate offset (~+38 µs/day); twin-paradox voyages against closed-form answers. |

The core test suite checks hundreds of known date conversions (from standard reference tables) across these calendars, plus **property tests**: date → instant → date round-trips for every calendar, and format render → parse round-trips.

> Note: we won't bundle JPL's high-precision ephemerides. The Solar System preset uses mean elements plus our own integrator, so it's "accurate enough to reproduce real calendars", not navigation-grade.

---

## 13. Delivery phases

Each phase ends with something usable.

| Phase | Scope | Usable outcome |
|---|---|---|
| **0. Foundations** | Rust workspace, `Instant` and rationals, SQLite schema and migrations, Tauri shell, CI. | App opens, creates and saves an empty universe. |
| **1. Calendar core** | Fixed fundamental unit, unit tree, groupings, members, parallel cycles, arithmetic rules, multiple eras, epoch anchoring and pre-epoch dates, expression language + builder, formats (render and parse), converter, calendar grid. Gregorian/Julian/Islamic-tabular/Hebrew presets on a fixed clock. | Design and convert arithmetic calendars. |
| **2. Bodies & Kepler** | Body editor, orbital elements, Keplerian (two-body) propagation, spin models, event finder, derived cycles, 2D orrery, rule suggester, surface sites, time zone schemes and zone rules. | Calendars derived from a planet's orbit and rotation, with local time. |
| **3. N-body** | REBOUND binding (IAS15/WHFast), ephemeris cache, reliability indicator, observational calendars, event-delimited units, sky-based zone rules, drift charts, Solar System validation. | Realistic long-term drift and observational calendars. |
| **4. Proper time & multiple systems** | Galactic frame, SBT, surface and orbital observers, clock tables, calendars bound to clocks, converter showing proper-time differences. | Compare calendars across planets and systems. |
| **5. Ships** | In-system thrust trajectories, interstellar relativistic legs, handoff, ship clocks, light-delay queries, timeline view. | Model voyages and the time dilation they cause. |
| **6. Polish** | 3D orrery, JSON import/export, project compaction, performance, documentation, example fictional universes. | Shareable, presentable app. |

**Why calendar core first:** it has no dependency on physics, while every physics feature (derived cycles, observational units, rule suggestion, drift charts) needs a calendar engine to be useful. The Earth presets also give the engine a strong test oracle from day one, so the foundations everything else rests on are verified before the harder numerical work starts.

---

## 14. Risks and open questions

### Risks

1. **Scope.** N-body physics plus relativistic trajectories across multiple systems is a lot of work. The phasing keeps each stage usable on its own.
2. **Hebrew-calendar-level complexity** is what drives expression-language features. If we keep that language small, we keep it safe and fast, but some exotic calendars may need custom functions.
3. **Parsing with arbitrary formats** can be ambiguous or impossible to invert. Mitigated by the invertibility check and the render-only fallback.
4. **Chaotic dynamics** make far-future dates in some systems meaningless. Mitigated by the reliability indicator.
5. **SQLite as the primary format** isn't diff-friendly. Mitigated by JSON export.
6. **REBOUND build on Windows.** It's developed mainly on Linux/macOS; compiling it with MSVC via `cc` may need small patches. Checked early in phase 3, with MinGW as a fallback.

---

## 15. Decisions log

| # | Question | Decision |
|---|---|---|
| 1 | Integrator sourcing | **Bind REBOUND** (C) via FFI, vendored and built by `build.rs`, behind an `Integrator` trait. |
| 2 | Absolute timeline zero | **Hidden technical zero.** GCT is never shown to users; it only equates points across calendars. Each calendar's epoch is the start of its **first defined era**, calendars can have multiple eras, and dates before the epoch are fully supported. |
| 3 | Time zones | **All schemes, chosen per body:** planetary standard, per-site solar time, named zones, and zones with historical and seasonal rules. Schemes can be mixed on one body. |
| 4 | Licensing | **Personal use only.** No distribution, so REBOUND's GPL-3.0 licence imposes no constraints. |
| 5 | Phase order | **Calendar core first** (left to Claude; reasoning in §13). |
