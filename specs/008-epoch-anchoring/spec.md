# Feature Specification: Epoch Anchoring and the Converter

**Feature Branch**: `design/008-epoch-anchoring`

**Created**: 2026-11-18

**Status**: Draft

**Input**: User description: "Epoch anchoring and the converter (issue #58, Phase 1. Calendar core). The second half of the 'first usable calendar' slice, after spec 007 (calendars in the project file). The first calendar's epoch sits at a fixed hidden point on the absolute timeline. Each later calendar is anchored by equating one of its dates with a date in an existing calendar. Then comes a converter screen that turns a date in one calendar into the same moment in another. The absolute timeline stays hidden (ADR 0002, constitution IV). Out of scope: formats, the calendar grid, multiple eras, presets. Use the calendar model's own terms (ADR 0009)."

**Issue**: #58 · **Depends on**: spec 004 (calendar engine basics), spec 006 (adjustment rules), spec 007 (calendars in the project file) · **Phase**: 1. Calendar core · **Knowledge**: [/calendar/eras-and-epoch.md](../../knowledge/calendar/eras-and-epoch.md), [/storage/schema.md](../../knowledge/storage/schema.md), [/product/ui-views.md](../../knowledge/product/ui-views.md), [/time/display-units.md](../../knowledge/time/display-units.md), [/time/timeline-and-frames.md](../../knowledge/time/timeline-and-frames.md), [/calendar/calendar-model.md](../../knowledge/calendar/calendar-model.md), [ADR 0002](../../knowledge/decisions/0002-hidden-absolute-timeline.md), [ADR 0008](../../knowledge/decisions/0008-physical-time-and-calendar-time.md), [ADR 0009](../../knowledge/decisions/0009-earth-neutral-names.md)

## User Scenarios & Testing *(mandatory)*

The examples use two test calendars. **Aster** is the calendar of specs 004–007: a sol of 88,775.244 seconds split into 20 bells of 100 pulses, an orbit of twelve spans counted in sols, with an adjustment rule. **Tallow** is a second, invented calendar: a watch of 30,000 seconds, a round of 7 watches, a cycle of 52 rounds.

### User Story 1 - Anchor a second calendar to the first (Priority: P1)

The owner has Aster in a universe; its epoch is the fixed hidden point, as it has been since spec 007. They add Tallow, and in the editor they say where Tallow sits in time by equating one of Tallow's dates with one of Aster's: "cycle 1, round 1, watch 1 of Tallow began at the start of orbit 412, span 3, sol 9 of Aster". They confirm, save, reopen, and the anchor is there exactly as they entered it.

**Why this priority**: Without an anchor, a second calendar has no relation to the first, and nothing can be compared. It is the core of the feature and of ADR 0002's "epochs are anchored by equating dates across calendars".

**Independent Test**: Define Aster and Tallow, anchor Tallow to an Aster date, save and reopen, and check the anchor shows the same two dates; then check that Tallow's anchoring date converts to the Aster date it was equated with.

**Acceptance Scenarios**:

1. **Given** a universe holding only Aster, **When** the owner opens Aster in the editor, **Then** it shows that Aster is the universe's reference calendar, with no anchor to enter, and no absolute-timeline value is shown.
2. **Given** a universe holding Aster, **When** the owner adds Tallow, enters its anchor (a Tallow date and an Aster date) and confirms, **Then** Tallow is saved with that anchor and the universe is marked as having unsaved changes.
3. **Given** Tallow anchored to Aster, **When** the owner saves, closes and reopens the universe, **Then** Tallow's editor shows the same anchor: the same calendar, the same two dates, as entered.
4. **Given** a third calendar, **When** the owner anchors it, **Then** they can equate it with any calendar already placed in time (Aster or Tallow), not only the first.
5. **Given** a universe file saved by spec 007 (calendars with no anchors), **When** the owner opens it, **Then** its first calendar becomes the reference calendar, every other calendar is anchored with its first date equal to the reference calendar's first date (the moment it already had), and the original file is copied beside it as the migration rule requires.

---

### User Story 2 - Convert a date between calendars (Priority: P1)

The owner opens the converter, picks a calendar, enters a date in it, and sees the same moment as a date in every other calendar of the universe.

**Why this priority**: The converter is what makes anchoring visible and useful: the first screen in the app that shows a calendar running, rather than only its definition.

**Independent Test**: With Aster and Tallow anchored, enter Tallow's anchoring date in the converter and check the Aster date it was equated with is shown; enter several other dates and check each against conversions worked out by hand or by the engine's own tests.

**Acceptance Scenarios**:

1. **Given** Tallow anchored at "cycle 1, round 1, watch 1 = orbit 412, span 3, sol 9", **When** the owner enters Tallow cycle 1, round 1, watch 1 in the converter, **Then** Aster is shown as orbit 412, span 3, sol 9, at its very start.
2. **Given** the same anchor, **When** the owner enters an Aster date, **Then** the Tallow date at the same moment is shown, including how far into its smallest unit the moment falls when it doesn't fall on a unit's start.
3. **Given** a date the calendar refuses (a span 13 in Aster, a watch 8 in a round of 7), **When** the owner enters it, **Then** no conversion is shown and the engine's reason is shown at the value it concerns.
4. **Given** a date before a calendar's epoch, **When** it is converted, **Then** it is shown with astronomical numbering (0, −1, …), as the engine already gives it.
5. **Given** a universe with one calendar, **When** the owner opens the converter, **Then** they can enter a date and see it checked, and are told there is no other calendar to convert into.

---

### User Story 3 - Anchors stay consistent when calendars change (Priority: P2)

The owner edits or deletes a calendar that other calendars are anchored to, or changes an anchor, and the universe never ends up with a calendar silently out of place.

**Why this priority**: Anchors link calendars together, so changes to one calendar reach others. Principle VII requires these effects to be visible, not silent.

**Independent Test**: Anchor Tallow to Aster; edit Aster so the Aster anchoring date no longer exists; check Tallow is reported with the reason. Try to delete Aster; check the outcome follows the deletion rule. Try to anchor Aster to Tallow; check it is refused as a loop.

**Acceptance Scenarios**:

1. **Given** Tallow anchored to Aster, **When** the owner tries to anchor Aster to Tallow (directly or through other calendars), **Then** it is refused with a message naming the loop.
2. **Given** Tallow anchored to an Aster date, **When** the owner edits Aster so that date no longer exists, **Then** Tallow is listed as unusable with the reason, and the converter does not offer it until its anchor is fixed.
3. **Given** Tallow anchored to Aster, **When** the owner tries to delete Aster, **Then** [NEEDS CLARIFICATION: deleting a calendar that others are anchored to: refuse and name the dependants, or re-anchor them to keep their moments?]

---

### Edge Cases

- **The reference calendar**: exactly one calendar in a universe has no anchor; its epoch is the fixed hidden point. The first calendar added to an empty universe becomes it.
- **Partial dates**: an anchoring date or converter date can stop at any unit from the era's unit down; missing smaller units take their first value, as the engine already does.
- **Moments between whole nanoseconds**: if an anchor puts a calendar's epoch, or a conversion lands, between two whole nanoseconds of the hidden timeline, the app says so and by how little it was moved (the engine's rounding report), never silently.
- **Out of range**: an anchor or date that would put a moment outside the timeline's range is refused with the engine's reason.
- **Dates before an epoch**: anchoring dates and converter dates can be before either calendar's epoch (astronomical numbering).
- **Units above the era's unit**: shown in converted dates (Aster's age and cohort) but never entered, as in spec 004.
- **Same calendar both sides**: a calendar can't be anchored to itself.
- **No Earth units**: anchors and conversions are shown only as calendar dates; no seconds, no raw timeline value, and no Earth unit appear outside a debug view.

## Requirements *(mandatory)*

### Functional Requirements

**Placing calendars in time**

- **FR-001**: Every universe with calendars MUST have exactly one **reference calendar**, whose epoch is the fixed hidden point on the absolute timeline. The first calendar added to an empty universe MUST become the reference calendar.
- **FR-002**: Every other calendar MUST be **anchored**: one of its dates is equated with a date in another calendar of the universe that is itself placed in time. The anchor MUST be entered and shown as those two dates and the other calendar's name, never as a timeline value.
- **FR-003**: An anchor MUST mean that the starts of the two dates are the same moment. The calendar's epoch follows from that and is never shown.
- **FR-004**: Anchoring MUST be refused, with the reason shown at the anchor, when either date is invalid in its calendar, when the anchor would form a loop, when the calendar is anchored to itself, or when the resulting moments fall outside the timeline's range.
- **FR-005**: [NEEDS CLARIFICATION: is an anchor kept as the equation, so a calendar moves when the calendar it is anchored to is edited, or is it resolved once to a fixed moment, so later edits to the other calendar don't move it?]
- **FR-006**: When a calendar's anchor can no longer be resolved (its anchoring date no longer exists after an edit, or a calendar it depends on is unusable), it MUST be listed as unusable with the reason and left out of the converter until fixed. Nothing is silently re-anchored.
- **FR-007**: A new calendar MUST start, in the editor, anchored to the reference calendar with its first date equal to the reference calendar's first date, shown so the owner can change it before confirming.

**The converter**

- **FR-008**: The app MUST have a converter screen, available when a universe with at least one usable calendar is open, where the owner picks a calendar and enters a date as a value for each unit from the era's unit down.
- **FR-009**: The converter MUST show the moment where that date starts as a date in every other usable calendar of the universe, with each unit named, including the units above the era's unit.
- **FR-010**: When the moment doesn't fall on the start of the smallest unit in a target calendar, the converter MUST show how far into that unit it falls, [NEEDS CLARIFICATION: as an exact fraction of the unit, a rounded decimal fraction of the unit, or only as "partway through"?], never in seconds.
- **FR-011**: A date the calendar refuses MUST be reported with the engine's reason at the value it concerns, and no conversion shown.
- **FR-012**: The converter MUST NOT show any absolute-timeline value or Earth unit.

**Storage**

- **FR-013**: Anchors MUST be stored in the project file as recorded in `/storage/schema.md` (updated by this feature's design PR), added by a numbered migration from schema version 2, and come back exactly as entered.
- **FR-014**: Migrating a version-2 file MUST keep every calendar at the moment it already had: the first calendar in the list becomes the reference calendar, and every other calendar is anchored with its first date equal to the reference calendar's first date.
- **FR-015**: Opening a file whose anchors can't be resolved (a loop, a missing calendar, a date that doesn't exist) MUST still open the universe and report each affected calendar as in FR-006.

**Terms and layering**

- **FR-016**: Labels and messages MUST use the calendar model's terms (ADR 0009), and the Earth-terms check MUST keep passing.
- **FR-017**: Working out epochs from anchors MUST use exact arithmetic (principle III) and live below the app and the UI.

### Key Entities

- **Reference calendar**: the one calendar with no anchor; its epoch is the fixed hidden point.
- **Anchor**: a calendar's link into time: the other calendar it is equated with, a date in this calendar, and a date in the other calendar, meaning the two dates start at the same moment.
- **Epoch (derived)**: where a calendar's first date starts on the hidden timeline, worked out from the reference calendar through the chain of anchors; never shown or entered.
- **Conversion**: a date entered in one calendar, and the same moment as a date in each other calendar, with how far into the smallest unit it falls.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The owner can anchor Tallow to Aster and convert a date between them in under 3 minutes, starting from a universe that holds both.
- **SC-002**: In a property test of at least 1,000 random anchors and dates, converting a date from one calendar to another and back gives the original date, and every anchor's two dates convert to each other.
- **SC-003**: Every kind of anchoring refusal (invalid date on either side, loop, self-anchor, out of range) is covered by a test, and 100% are refused with the reason shown; none is saved.
- **SC-004**: A version-2 file opens and migrates with its backup made, and every converted date in it is the same as it would have been before the migration.
- **SC-005**: Converting a date in a universe of 20 calendars, including one with a 1,000-member grouping, shows every result in under 1 second on the owner's machine.
- **SC-006**: No screen outside a debug view shows a timeline value or an Earth unit; the Earth-terms, architecture and privacy checks pass.

## Assumptions

- **Anchors to calendar dates only**: anchoring to timeline events or sky events (`/calendar/eras-and-epoch.md`) needs events and physics, and comes with those features.
- **One era**: calendars still have a single era (spec 007), so a calendar's epoch is its first era's start. Multiple eras and pre-epoch labelling are later features; dates before an epoch use astronomical numbering.
- **Dates entered value by value**: without formats, the converter and the anchor take a date as one value per unit, named by the unit, as the editor does. Formats come later.
- **No observers' clocks**: every calendar counts the absolute timeline's clock (spec 004's only clock), so a conversion is the same moment on one timeline. Observers' clocks and proper-time differences, which the full converter of `/product/ui-views.md` shows, come with physics.
- **Display calendar**: choosing the universe's display calendar (`/time/display-units.md`) is not part of this feature.
- **Knowledge update**: the design PR updates `/storage/schema.md` (anchor storage, schema version 3), `/calendar/eras-and-epoch.md` (the reference calendar and anchoring by calendar dates as built), and `/product/ui-views.md` (the minimal converter as the first step towards the full one), for the owner to re-verify.
- **Scope check (principle VIII)**: if planning shows the build PR would exceed about 30 minutes of review, the spec is split again (for example, anchoring first, then the converter) before the design PR is opened.
