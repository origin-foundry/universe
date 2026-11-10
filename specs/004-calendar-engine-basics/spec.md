# Feature Specification: Calendar Engine Basics

**Feature Branch**: `design/004-calendar-engine-basics`

**Created**: 2026-11-10

**Status**: Approved by the owner, 2026-11-10

**Input**: User description: "Calendar engine basics (issue #37, Phase 1 spec 004). The calendar engine (uni-calendar) can represent a calendar built on a fixed fundamental unit (a constant length on its clock), with a unit tree of subdivisions and fixed-count groupings (including named members) and a single era whose epoch is anchored on the timeline. It converts exactly in both directions between instants and dates, before and after the epoch, and reports invalid dates with the reason instead of wrapping them. Out of scope: leap rules and the expression language, parallel cycles, multiple eras, intercalary members, event-delimited units, formats, presets, and any calendar editing UI."

**Issue**: #37 · **Phase**: 1. Calendar core · **Knowledge**: [/calendar/calendar-model.md](../../knowledge/calendar/calendar-model.md), [/calendar/unit-kinds.md](../../knowledge/calendar/unit-kinds.md), [/calendar/conversion.md](../../knowledge/calendar/conversion.md), [/calendar/eras-and-epoch.md](../../knowledge/calendar/eras-and-epoch.md), [/time/timeline-and-frames.md](../../knowledge/time/timeline-and-frames.md), [/architecture/crate-layout.md](../../knowledge/architecture/crate-layout.md)

## Clarifications

### Session 2026-11-10

- Q: Should spec 004 be the calendar engine only, with nothing new on screen or in the project file? → A: Engine only, like spec 002: the engine and its tests, no UI, no storage. Storing and editing calendars comes with the first feature that lets the owner create one.
- Q: How should years before the epoch be numbered, given the epoch starts year 1? → A: Astronomical numbering: the year before year 1 is year 0, then −1, −2, and so on. The per-calendar choice of a counting-down pre-epoch era (with or without year zero), already described in `/calendar/eras-and-epoch.md`, comes with the multiple-eras feature.
- Q: When a unit's start falls between two nanoseconds, what should converting that date to an instant do? → A: The engine keeps boundaries exact internally; the instant returned is the first whole nanosecond at or after the true start (so it still falls in that date), and the result says it was rounded and by exactly how much.
- Q: Where should each unit start counting within its parent? → A: A per-unit setting: each unit below the era's unit has a first number (any whole number). Defaults are 1 for months and sols (units above or at the fundamental unit) and 0 for subdivisions such as hours and minutes. The era's unit is numbered by the era.
- Q: (Owner, in spec review) The fundamental unit has a fixed length, but a real day isn't a fixed length and changes over time; is that covered? → A: Yes. A calendar's day is whatever length its definer chooses and does not have to match the planet's real rotation; the physical day is a separate thing the physics computes. This spec builds only fixed-length units. Calendars that choose to follow the sky (event-delimited units, Phase 3) are an option, never a requirement. Recorded in FR-002a and the ground rules below.
- Q: (Owner, in spec review) Units above the year (decade, century, millennium) have nowhere to go if the era numbers the top-level unit; how should they be handled? → A: The era picks a level: it numbers a chosen unit (the year), and fixed-count groupings may sit above it (decade = 10 years, century = 10 decades). Each unit above the era's unit is numbered by its count from the epoch, so century 1 holds years 1–100 and century 0 holds years −99 to 0. The design PR changes `/calendar/eras-and-epoch.md` from "the top-level unit" to "a chosen unit", for the owner to re-verify.
- Q: (Owner, in spec review) Physics models true astronomical time while human calendars don't fully respect physics; is it right that physical time is recorded on GCT and calendars, with their faults, are mapped onto it, and how do we stop this causing confusion later? → A: Yes. The design PR records it as ground rules in a new ADR 0008, "Physical time and calendar time", with matching glossary terms, verified by the owner with the other knowledge changes. The draft rules are under "Physical time and calendar time" below; this spec follows them.

## User Scenarios & Testing *(mandatory)*

This feature has no screen. Its users are the **features built on it**: leap rules, cycles, eras, formats, the converter and the calendar grid later in Phase 1, and the physics-derived calendars of Phase 2. The **owner** sees its effect indirectly: every date the app will ever show goes through this engine, so it must be exact and must refuse impossible dates.

Examples below use a made-up calendar, **Aster**, so nothing depends on Earth:

- fundamental unit: the **sol**, 88,775.244 seconds long;
- a **sol** is subdivided into 20 **hours**, each hour into 100 **minutes**;
- a **year** is a grouping of 12 **months** with fixed lengths `[56, 56, 55, 56, 56, 55, 56, 56, 55, 56, 56, 55]` (668 sols), named Prime, Second, … Twelfth;
- a **decade** is a grouping of 10 years, and an **age** a grouping of 10 decades;
- months and sols count from 1, hours and minutes from 0 (the defaults);
- one era, "After Landing", numbering **years**, starting at year 1, month 1, sol 1; the start of that sol is placed at a chosen instant on the hidden timeline. Decades and ages are numbered by their count from the epoch: decade 1 holds years 1–10, age 1 holds years 1–100.

### User Story 1 - Define a calendar (Priority: P1)

Later features need to describe a calendar in the engine: its fundamental unit and length, the units above it (groupings with fixed child counts and optional member names) and below it (equal subdivisions), and its era and epoch. The engine checks the definition and rejects ones that can't work, saying why.

**Why this priority**: nothing can be converted until a calendar can be described. Every later Phase 1 feature adds to this definition.

**Independent Test**: build the Aster calendar and a few deliberately broken ones, and check that Aster is accepted and each broken one is refused with a reason that names the faulty part.

**Acceptance Scenarios**:

1. **Given** the Aster definition, **When** it's checked, **Then** it's accepted, and the engine reports a year as 668 sols and a sol as 2,000 minutes.
2. **Given** a definition whose fundamental unit has zero or negative length, **When** it's checked, **Then** it's refused, naming the unit and the reason.
3. **Given** a grouping with no members, a member with a length of zero or less, or a subdivision into fewer than one part, **When** it's checked, **Then** it's refused, naming the unit (and member) at fault.
4. **Given** two units, or two members of one grouping, with the same name, **When** it's checked, **Then** it's refused, naming the duplicate.
5. **Given** a definition whose era numbers a unit that isn't in the calendar, or that is below the fundamental unit, **When** it's checked, **Then** it's refused, saying why.
6. **Given** Aster changed so hours count from 1 instead of 0, **When** the epoch is converted to a date, **Then** the hour is 1, and hour 0 is refused as outside the range 1 to 20.
7. **Given** Aster with its sol set to any other length (say exactly 100,000 seconds), whatever the planet's real rotation, **When** it's checked, **Then** it's accepted with no warning: a calendar's units are the definer's choice (FR-002a).

---

### User Story 2 - Instant to date (Priority: P1)

Given any instant on the timeline, later features need the date it falls on in a calendar: the number of the era's unit (the year) and of every unit below it down to the smallest subdivision, the numbers of the units above it (decade, age), and the exact remainder below the smallest unit.

**Why this priority**: showing any moment to the user means turning it into a date. With Story 1 it's the minimum useful engine.

**Independent Test**: convert instants at, after and before the epoch in the Aster calendar and compare with dates worked out by hand.

**Acceptance Scenarios**:

1. **Given** the epoch instant, **When** it's converted, **Then** the date is year 1, Prime (month 1), sol 1, hour 0, minute 0, with no remainder, in decade 1 and age 1.
2. **Given** the instant exactly 668 sols after the epoch, **When** it's converted, **Then** the date is year 2, month 1, sol 1.
3. **Given** the instant one nanosecond before the epoch, **When** it's converted, **Then** the date is year 0, Twelfth, sol 55, hour 19, minute 99, with a remainder of one minute less one nanosecond, in decade 0 and age 0.
4. **Given** the start of year 101, **When** it's converted, **Then** it's in decade 11 and age 2; the start of year −100 is in decade −10 and age −1.
5. **Given** an instant part-way through a minute, **When** it's converted, **Then** the remainder is the exact time since the start of that minute, and adding it to the minute's start gives back the original instant exactly.
6. **Given** an instant whose date would need a year number outside the supported range, **When** it's converted, **Then** an out-of-range error is reported, never a wrapped value.

---

### User Story 3 - Date to instant (Priority: P1)

Given a date in a calendar, later features need the instant it starts at, and must be told clearly when a date doesn't exist.

**Why this priority**: entering a date, the converter and every rule anchored on a date depend on it; principle VII requires impossible dates to be reported, not wrapped.

**Independent Test**: convert valid Aster dates to instants and back, and try invalid ones, checking each is refused with the reason.

**Acceptance Scenarios**:

1. **Given** year 1, Prime, sol 1, **When** it's converted, **Then** the result is exactly the epoch instant.
2. **Given** any valid date, **When** it's converted to an instant and back, **Then** the same date comes back, with no remainder.
3. **Given** sol 56 of Third (a 55-sol month), **When** it's converted, **Then** it's refused, saying Third has only 55 sols.
4. **Given** month 13, sol 0, hour 20 or minute −1, **When** it's converted, **Then** it's refused, naming the unit and its valid range.
5. **Given** a date that gives only some units (for example year and month), **When** it's converted, **Then** the missing smaller units are taken as their first value, so the result is the start of that month.
6. **Given** a date, **When** it's entered, **Then** it starts at the era's unit (the year); decades and ages are worked out from the year, not entered.

---

### User Story 4 - Exact conversion everywhere (Priority: P2)

The engine must stay exact across the whole range later features need, including far before the epoch and with fundamental-unit lengths that aren't whole seconds.

**Why this priority**: Stories 2 and 3 cover the behaviour; this story proves it holds everywhere, which constitution principles III and VI require.

**Independent Test**: run property tests over random calendars, instants and dates, in both directions, across the whole supported range.

**Acceptance Scenarios**:

1. **Given** random valid calendars and random instants within ±100,000 Aster years of the epoch, **When** each instant is converted to a date and back (start of date plus remainder), **Then** the original instant comes back exactly.
2. **Given** random valid dates, **When** each is converted to an instant and back, **Then** the same date comes back.
3. **Given** a fundamental-unit length whose unit boundaries don't fall on whole nanoseconds, **When** dates are converted, **Then** each date's instant is the first whole nanosecond at or after its exact start, the result says it was rounded and by how much, and converting that instant back gives the same date.

### Edge Cases

- **Before the epoch**: dates before the epoch use the same unit lengths running backwards (proleptic); fixed numbers are negative; ordering and arithmetic behave exactly as after the epoch.
- **Boundaries**: an instant exactly on a unit boundary belongs to the unit that starts there, never the one that ends there.
- **Boundaries between nanoseconds**: when a unit's exact start isn't a whole nanosecond (a sol split into 7 hours), date-to-instant returns the next whole nanosecond and says so (FR-014); the remainder from instant-to-date is then an exact fraction, not a whole number of nanoseconds.
- **Range limits**: instants and dates that would need numbers outside the supported range are errors, not wrapped or clipped.
- **Calendars with only a fundamental unit**: a calendar with no groupings or subdivisions is valid; its dates are just the fixed number in era terms.
- **Single-member groupings**: a grouping with one member (a "year" of one 400-sol "month") is valid.
- **Very long and very short units**: a fundamental unit as short as one nanosecond or as long as a million years must convert exactly, within the overall range.

## Requirements *(mandatory)*

### Functional Requirements

**Defining a calendar**

- **FR-001**: The engine MUST represent a calendar with: a name; a clock; exactly one fundamental unit of fixed length; zero or more groupings stacked above it in a single chain up to a top-level unit; zero or more subdivisions stacked below it in a single chain; and one era, which numbers a chosen unit at or above the fundamental unit (the **era's unit**, such as the year). Groupings may sit above the era's unit (decade, century). This is the subset of `/calendar/calendar-model.md` and `/calendar/unit-kinds.md` that this feature builds.
- **FR-002**: The fundamental unit's length MUST be an exact positive amount of time on the calendar's clock, allowing fractions of a second (no floating point).
- **FR-002a**: The fundamental unit's length MUST be whatever the definer chooses. The engine MUST NOT require, check or warn that it (or any other unit) matches a physical rotation, orbit or other phenomenon: calendars don't have to be astronomically accurate.
- **FR-003**: A grouping MUST list its members in order, each with a positive whole-number count of the unit below it and an optional name. Member counts are fixed (leap rules come later).
- **FR-004**: A subdivision MUST split its parent into a whole number (one or more) of equal children.
- **FR-005**: Each unit MUST have a name, unique within the calendar; member names MUST be unique within their grouping.
- **FR-005a**: Each unit below the era's unit MUST have a **first number**: the number of its first instance within its parent, any whole number (positive, zero or negative). If not given, it defaults to 1 for units at or above the fundamental unit (months, sols) and 0 for subdivisions (hours, minutes). Valid values of that unit then run from the first number up to first number + count − 1.
- **FR-006**: The engine MUST check a definition before use and refuse invalid ones with an error naming the unit or member at fault and the reason (FR-002 to FR-005a), including an era whose unit isn't in the calendar or is below the fundamental unit.
- **FR-007**: For this feature the clock MUST be the absolute timeline itself (GCT); system and observer clocks come with Phase 4. The definition MUST keep the clock as an explicit part of the calendar so later features can change it without reshaping calendars.

**Epoch and era**

- **FR-008**: The calendar's single era MUST define the epoch: the start of the era's first date (the first instance of the era's unit and of every unit below it) is fixed number 0, and is placed at a chosen instant on the timeline.
- **FR-008a**: Each unit above the era's unit MUST be numbered by its count from the epoch, using the same astronomical numbering as the era (FR-009): the one that starts at the epoch is 1, the one before it 0, then −1, and so on. In Aster, decade 1 holds years 1–10 and decade 0 holds years −9 to 0.
- **FR-009**: Dates before the epoch MUST be valid, with the calendar's units running backwards unchanged. Top-level units use astronomical numbering: the era starts at 1, and the ones before it are 0, −1, −2, and so on, with no gap.

**Conversion**

- **FR-010**: The engine MUST convert an instant to a date: the number of the era's unit and of every unit below it down to the smallest subdivision, the number of every unit above the era's unit (FR-008a), and the exact remainder of time below the smallest unit. A date given for conversion to an instant starts at the era's unit; units above it are derived, not entered.
- **FR-011**: The engine MUST convert a date to the instant at which it starts. A date MAY stop at any unit; smaller units are then taken as their first value.
- **FR-012**: The engine MUST convert an instant to its fixed number and a fixed number to its starting instant, as the backbone of FR-010 and FR-011.
- **FR-013**: Any date with a value outside its unit's range (including a member count that doesn't fit that member, such as sol 56 of a 55-sol month) MUST be refused with an error naming the unit, the value given and the valid range. Dates are never wrapped or normalised.
- **FR-014**: Conversions MUST be exact: no floating point, and unit boundaries kept as exact fractions inside the engine. The only rounding is when a date's exact start falls between two nanoseconds: the instant returned is then the first whole nanosecond at or after that start, and the result MUST say it was rounded and give the exact amount. Instant-to-date never rounds; its remainder (FR-010) is measured from the exact start.
- **FR-015**: Every operation whose result would leave the supported range MUST report an out-of-range error.
- **FR-016**: Conversion of a fixed-length calendar MUST take constant time, independent of how far the date is from the epoch (the supercycle strategy of `/calendar/conversion.md` in its simplest form: one instance of the era's unit is the repeating cycle).

**Quality**

- **FR-017**: The engine MUST live in the calendar component (`uni-calendar`), depending only on the components `/architecture/crate-layout.md` allows, and MUST NOT use floating point (it's an exact crate).
- **FR-018**: The engine MUST be written test-first, with tests against hand-worked dates for Aster and at least one other calendar of different shape, and round-trip property tests in both directions (principle VI).
- **FR-019**: Errors MUST be structured (which unit, which value, which range, which reason) so later features can show a clear message.

### Key Entities

- **Calendar**: name, clock, fundamental unit, unit chain, era.
- **Fundamental unit**: the smallest unit counted whole (e.g. the sol); fixed exact length.
- **Grouping**: a unit made of an ordered list of members, each a fixed count of the unit below, optionally named (e.g. year → months, month → sols).
- **Subdivision**: a unit split into *n* equal children (e.g. sol → 20 hours).
- **Era**: numbers a chosen unit (the era's unit, such as the year); the first (and here only) era defines the epoch. Units above the era's unit are numbered by their count from the epoch.
- **First number**: per unit below the era's unit, the number of its first instance within its parent (defaults 1, or 0 for subdivisions).
- **Epoch anchor**: the instant on the timeline where fixed number 0 starts.
- **Fixed number**: whole count of fundamental units from the epoch; negative before it.
- **Date**: a value for the era's unit and each unit below it down to some unit (with the numbers of the units above the era's unit when produced from an instant), plus (when produced from an instant) the exact remainder below the smallest unit given.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of hand-worked test dates convert correctly in both directions, at, after and before the epoch.
- **SC-002**: Round-trip property tests run at least 10,000 random cases per property with no failures, across ±100,000 years of the epoch.
- **SC-003**: 100% of the invalid dates and invalid definitions in the tests are refused with an error naming the unit and reason; none produces a date or instant.
- **SC-004**: A single conversion takes under 10 microseconds on the owner's machine, whether the date is near the epoch or 100,000 years away.
- **SC-005**: The calendar engine contains no floating-point code, as confirmed by the architecture check.

## Assumptions

- **Engine only**: no screen, no storage, no editing UI in this feature (confirmed in Clarifications). The converter, calendar grid and calendar editor are later Phase 1 features; storing calendars in the project file comes with the first feature that lets the owner create one.
- **Clock**: only GCT exists until Phase 4, so calendars count GCT for now (FR-007).
- **Fixed fundamental unit only**: the fundamental unit here is the fixed kind from `/calendar/calendar-model.md`: a constant length the definer chooses, with no tie to the planet's real rotation (FR-002a). The physical day, which varies and drifts, is computed separately by the physics (Phase 2). Calendars that choose to follow the sky use event-delimited units (Phase 3); that is optional.
- **Epoch anchoring**: the other anchoring methods in `/calendar/eras-and-epoch.md` (a date in another calendar, a timeline event, a sky event) need features that don't exist yet; for now the anchor is an instant on the hidden timeline, never shown to the user (principle IV).
- **Pre-epoch labelling comes later**: the counting-down pre-epoch era ("Before Founding", with or without a year zero) and the per-calendar choice between it and astronomical numbering, described in `/calendar/eras-and-epoch.md`, are part of the multiple-eras feature.
- **Shape**: groupings and subdivisions each form a single chain (year → month → sol → hour → minute). Branching trees, parallel cycles, intercalary members and event-delimited units come later.
- **Knowledge update**: the per-unit first number (FR-005a), the next-nanosecond rule (FR-014) and the era numbering a chosen unit with units above it (FR-001, FR-008a) aren't in the bundle yet; the design PR adds them to `/calendar/unit-kinds.md`, `/calendar/conversion.md` and `/calendar/eras-and-epoch.md`, and adds ADR 0008 and the glossary changes (see "Physical time and calendar time"), for the owner to re-verify.
- **Display is separate**: a unit's first number (FR-005a) is part of the date's meaning; how numbers are shown (padding, names, ordinals) belongs to the later formats feature.
- **No presets**: Earth presets need leap rules, so the independent-implementation oracle of principle VI applies from the first preset spec; this feature uses hand-worked dates and property tests.

## Physical time and calendar time

Ground rules agreed in spec review, to be recorded as ADR 0008 (with glossary updates) in this spec's design PR and verified by the owner there. Every later spec states which side of this line each concept it uses is on.

1. **Physical time** is GCT and the clocks derived from it (system time, an observer's proper time). Physics works only in these.
2. **Real phenomena** (a planet's rotation, its orbit, sunrise, a lunation) are events and physical cycles that the physics finds and places on the timeline. They are facts.
3. **A calendar is a human convention** laid over one clock, tied to the timeline only by that clock and its epoch anchor. It never alters physics, and physics never depends on it (principle V).
4. **Calendars don't have to be astronomically accurate.** A calendar's day, month or year is whatever its definer chooses. Matching the sky is optional: a unit can be any fixed length (perhaps fitted from physics as a starting point, then frozen), rule-adjusted (leap rules), or event-delimited (follows the sky), and the first is as valid as the last.
5. **Drift between a calendar and the sky is never an error.** The engine never refuses or warns about it; when physics exists, it is shown as information (drift charts, the rule suggester) for users who care.
6. **The app says so plainly.** When calendar editing reaches the UI, it tells the user that units are their choice and needn't match the sky, and never presents astronomical accuracy as a requirement.
7. **Words**: "day", "year" and so on inside a calendar mean calendar units; physical ones are always named as such ("rotation", "solar day at site X"). The glossary's **Cycle** becomes **Physical cycle**, so it can't be confused with a calendar's **parallel cycle**. "Astronomical year numbering" (year 0, −1, …) is only a numbering convention, not physics.

This spec sits entirely on the calendar side: a fixed-length fundamental unit of the definer's choosing (rule 4, FR-002a), mapped onto GCT by an epoch anchor (rule 3).
