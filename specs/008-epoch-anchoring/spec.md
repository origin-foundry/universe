# Feature Specification: Epoch Anchoring and the Converter

**Feature Branch**: `design/008-epoch-anchoring`

**Created**: 2026-11-18

**Status**: Approved by the owner, 2026-11-18; amended in design review the same day (terms, and anchoring the first calendar later)

**Input**: User description: "Epoch anchoring and the converter (issue #58, Phase 1. Calendar core). The second half of the 'first usable calendar' slice, after spec 007 (calendars in the project file). The first calendar's epoch sits at a fixed hidden point on the absolute timeline. Each later calendar is anchored by equating one of its dates with a date in an existing calendar. Then comes a converter screen that turns a date in one calendar into the same moment in another. The absolute timeline stays hidden (ADR 0002, constitution IV). Out of scope: formats, the calendar grid, multiple eras, presets. Use the calendar model's own terms (ADR 0009)."

**Issue**: #58 · **Depends on**: spec 004 (calendar engine basics), spec 006 (adjustment rules), spec 007 (calendars in the project file) · **Phase**: 1. Calendar core · **Knowledge**: [/calendar/eras-and-epoch.md](../../knowledge/calendar/eras-and-epoch.md), [/storage/schema.md](../../knowledge/storage/schema.md), [/product/ui-views.md](../../knowledge/product/ui-views.md), [/time/display-units.md](../../knowledge/time/display-units.md), [/time/timeline-and-frames.md](../../knowledge/time/timeline-and-frames.md), [/calendar/calendar-model.md](../../knowledge/calendar/calendar-model.md), [ADR 0002](../../knowledge/decisions/0002-hidden-absolute-timeline.md), [ADR 0008](../../knowledge/decisions/0008-physical-time-and-calendar-time.md), [ADR 0009](../../knowledge/decisions/0009-earth-neutral-names.md)

## Clarifications

### Session 2026-11-18

- Q: If a calendar that another is anchored to is later edited, does the anchored calendar move with it? → A: By default no: an anchor fixes the moment when it is confirmed, and later edits to the other calendar don't move this one. When defining the anchor, the owner can instead choose to keep the equation, so the calendar follows the other calendar's edits and its anchoring date always starts at the other calendar's date.
- Q: What happens to calendars that keep an equation with a calendar the owner deletes? → A: The delete confirmation names them; on confirming, their anchors turn into anchors that fix the moment, so they stay exactly where they are. Their anchors still show the dates as entered, noting that the other calendar is gone.
- Q: How does the converter show a moment that falls partway through the smallest unit of a target calendar? → A: At a precision the owner picks with a switch on the converter screen, applying to every result: "partway through" with no amount (the default), a decimal fraction of the unit, or the exact fraction. The switch keeps its setting while the app is open and starts at "partway" at the next launch; an app-wide preferences screen can take it over later.

### Design review 2026-11-18

- Q: What are the calendars in an anchor called? → A: The calendar being placed is the **anchored calendar**; the calendar it's equated with is its **base calendar**. The two dates are the **anchor date** (in the anchored calendar) and the **base date** (in the base calendar). Chains of bases end at the universe's **reference calendar**, a term for the design and code; the app doesn't need to show it.
- Q: Can the first calendar, which has no anchor, be anchored later? → A: Yes. The first calendar sits silently at the hidden zero, with nothing to show or enter, because it's clear it needs no anchor. The owner can later anchor it to a calendar defined after it, like any other calendar, as long as that doesn't make a loop. After that, no calendar is at the hidden zero, and that's fine: the hidden zero means nothing to the owner.
- Q: How does the owner choose a base calendar? → A: From a list of the universe's other calendars. Calendars that can't be a base (one that follows the anchored calendar, or one that can't be placed) are listed but greyed out with the reason, so it's clear why they aren't available.
- Q: What feedback does the anchor give while the owner types? → A: Each date is checked by the engine as it's typed, and an invalid value is flagged at once at its field. Under the fields, a sentence reads the anchor back in words, naming members where they have names ("Tallow's cycle 1, round 1, watch 1 began at the start of Aster's orbit 412, span 3 (Third), sol 9.").

## User Scenarios & Testing *(mandatory)*

The examples use two test calendars. **Aster** is the calendar of specs 004–007: a sol of 88,775.244 seconds split into 20 bells of 100 pulses, an orbit of twelve spans counted in sols, with an adjustment rule. **Tallow** is a second, invented calendar: a watch of 30,000 seconds, a round of 7 watches, a cycle of 52 rounds.

### User Story 1 - Anchor a second calendar to the first (Priority: P1)

The owner has Aster in a universe; its epoch is the fixed hidden point, as it has been since spec 007. They add Tallow, and in the editor they say where Tallow sits in time by equating one of Tallow's dates with one of Aster's: "cycle 1, round 1, watch 1 of Tallow began at the start of orbit 412, span 3, sol 9 of Aster". They confirm, save, reopen, and the anchor is there exactly as they entered it.

**Why this priority**: Without an anchor, a second calendar has no relation to the first, and nothing can be compared. It is the core of the feature and of ADR 0002's "epochs are anchored by equating dates across calendars".

**Independent Test**: Define Aster and Tallow, anchor Tallow to an Aster date, save and reopen, and check the anchor shows the same two dates; then check that Tallow's anchor date converts to its base date in Aster.

**Acceptance Scenarios**:

1. **Given** a universe holding only Aster, **When** the owner opens Aster in the editor, **Then** it shows that Aster isn't anchored, with nothing to enter, and no absolute-timeline value is shown.
2. **Given** a universe holding Aster, **When** the owner adds Tallow, enters its anchor (Aster as the base calendar, an anchor date in Tallow and a base date in Aster) and confirms, **Then** Tallow is saved with that anchor and the universe is marked as having unsaved changes.
3. **Given** Tallow anchored to Aster, **When** the owner saves, closes and reopens the universe, **Then** Tallow's editor shows the same anchor: the same base calendar, the same two dates, as entered.
4. **Given** a third calendar, **When** the owner anchors it, **Then** they can choose any calendar already placed in time (Aster or Tallow) as its base, not only the first.
6. **Given** Aster unanchored and Tallow anchored to it, fixing the moment, and a third calendar, Wick, anchored to Tallow, **When** the owner anchors Aster to Wick, **Then** Aster is placed from Wick, Tallow and Wick stay where they were, and no calendar is left unanchored.
5. **Given** a universe file saved by spec 007 (calendars with no anchors), **When** the owner opens it, **Then** its first calendar stays unanchored (the reference calendar), every other calendar is anchored to it with the first dates of both as anchor date and base date (the moment it already had), and the original file is copied beside it as the migration rule requires.

---

### User Story 2 - Convert a date between calendars (Priority: P1)

The owner opens the converter, picks a calendar, enters a date in it, and sees the same moment as a date in every other calendar of the universe.

**Why this priority**: The converter is what makes anchoring visible and useful: the first screen in the app that shows a calendar running, rather than only its definition.

**Independent Test**: With Aster and Tallow anchored, enter Tallow's anchor date in the converter and check its base date in Aster is shown; enter several other dates and check each against conversions worked out by hand or by the engine's own tests.

**Acceptance Scenarios**:

1. **Given** Tallow anchored at "cycle 1, round 1, watch 1 = orbit 412, span 3, sol 9", **When** the owner enters Tallow cycle 1, round 1, watch 1 in the converter, **Then** Aster is shown as orbit 412, span 3, sol 9, at its very start.
2. **Given** the same anchor, **When** the owner enters an Aster date, **Then** the Tallow date at the same moment is shown, marked as partway through its smallest unit when the moment doesn't fall on that unit's start.
3. **Given** a result partway through a unit, **When** the owner switches the converter's precision to decimal or exact, **Then** every result shows how far into its smallest unit the moment falls, as a decimal fraction (marked when rounded) or an exact fraction of that unit.
4. **Given** a date the calendar refuses (a span 13 in Aster, a watch 8 in a round of 7), **When** the owner enters it, **Then** no conversion is shown and the engine's reason is shown at the value it concerns.
5. **Given** a date before a calendar's epoch, **When** it is converted, **Then** it is shown with astronomical numbering (0, −1, …), as the engine already gives it.
6. **Given** a universe with one calendar, **When** the owner opens the converter, **Then** they can enter a date and see it checked, and are told there is no other calendar to convert into.

---

### User Story 3 - Anchors stay consistent when calendars change (Priority: P2)

The owner edits or deletes a calendar that other calendars are anchored to, or changes an anchor, and the universe never ends up with a calendar silently out of place.

**Why this priority**: Anchors link calendars together, so changes to one calendar reach others. Principle VII requires these effects to be visible, not silent.

**Independent Test**: Anchor Tallow to Aster each way (fixing the moment, keeping the equation); edit Aster before the base date and so the base date no longer exists; check Tallow stays put or follows, or is reported with the reason, as its anchor kind says. Try to delete Aster; check the outcome follows the deletion rule. Try to anchor a calendar to one that follows it; check it is refused as a loop.

**Acceptance Scenarios**:

1. **Given** Tallow keeps an equation with Aster and a third calendar, Wick, keeps an equation with Tallow (so Wick follows Tallow), **When** the owner changes Tallow's anchor to have Wick as its base, of either kind, **Then** it is refused with a message naming the loop. (Fixing the moment doesn't help: Tallow's move would move Wick, so the two dates wouldn't match even as they were confirmed.)
2. **Given** Tallow anchored to Aster, **When** the owner edits Aster so that the base date no longer exists, **Then** if Tallow's anchor fixes the moment, Tallow stays where it was and its editor shows the Aster date that moment now falls at; if it keeps the equation, Tallow is listed as unusable with the reason, and the converter does not offer it until its anchor is fixed.
3. **Given** Tallow anchored to Aster with an anchor that keeps the equation, **When** the owner lengthens one of Aster's spans before the base date, **Then** Tallow moves so that its anchor date still starts at the base date, and conversions show it.
4. **Given** Tallow anchored to Aster, **When** the owner tries to delete Aster, **Then** the confirmation names every calendar that follows Aster (keeps an equation with it); on confirming, Aster is deleted, those anchors turn into anchors that fix the moment, every other calendar stays at the same moment (conversions between them are unchanged), and each anchor whose base was Aster shows its dates as entered, noting Aster is gone.

---

### Edge Cases

- **The reference calendar**: at most one calendar in a universe has no anchor; its epoch is the fixed hidden point. The first calendar added to an empty universe becomes it, silently. Anchoring it later, or deleting it, leaves the others where they are (FR-001a, FR-006a); new calendars are then anchored to any remaining calendar.
- **Partial dates**: an anchor date, base date or converter date can stop at any unit from the era's unit down; missing smaller units take their first value, as the engine already does.
- **Moments between whole nanoseconds**: if an anchor puts a calendar's epoch, or a conversion lands, between two whole nanoseconds of the hidden timeline, the app says so (the engine's rounding report), never silently, without giving the amount in seconds.
- **Out of range**: an anchor or date that would put a moment outside the timeline's range is refused with the engine's reason.
- **Dates before an epoch**: anchor dates, base dates and converter dates can be before either calendar's epoch (astronomical numbering).
- **Units above the era's unit**: shown in converted dates (Aster's age and cohort) but never entered, as in spec 004.
- **Same calendar both sides**: a calendar can't be anchored to itself.
- **No Earth units**: anchors and conversions are shown only as calendar dates; no seconds, no raw timeline value, and no Earth unit appear outside a debug view.

## Requirements *(mandatory)*

### Functional Requirements

**Placing calendars in time**

- **FR-001**: A universe MUST have at most one calendar with no anchor, the **reference calendar**, whose epoch is the fixed hidden point on the absolute timeline. The first calendar added to an empty universe MUST become the reference calendar, silently: the app shows only that it isn't anchored, never the term or the hidden point.
- **FR-001a**: The owner MUST be able to anchor the reference calendar later, to any calendar that doesn't follow it (FR-004), like any other calendar. It then stops being the reference calendar, and no calendar is at the hidden zero; every calendar keeps its moment except the one being anchored and the calendars that follow it.
- **FR-002**: Every other calendar MUST be **anchored**: its **anchor date** is equated with a **base date** in its **base calendar**, another calendar of the universe that is itself placed in time. The anchor MUST be entered and shown as those two dates and the base calendar's name, never as a timeline value. Once anchored, a calendar always has an anchor; it can be changed but not removed.
- **FR-002a**: The base calendar MUST be chosen from a list of the universe's other calendars, never typed. Calendars that can't be the base (one that follows the anchored calendar, FR-004, or one that can't be placed, FR-006) MUST be listed but not choosable, each with the reason.
- **FR-002b**: While the owner types, each date of the anchor MUST be checked by its calendar, and an invalid value MUST be flagged at its field with the engine's reason, before the owner confirms. Under the dates, the editor MUST read the anchor back as a sentence naming both calendars and both dates, with member names where members have them. The anchor date can only be checked once the calendar's own definition is valid; until then the editor says so.
- **FR-003**: An anchor MUST mean that the starts of the two dates are the same moment. The calendar's epoch follows from that and is never shown.
- **FR-004**: Anchoring MUST be refused, with the reason shown at the anchor, when either date is invalid in its calendar, when the base calendar **follows** the anchored calendar (keeps an equation with it, directly or through other calendars that keep equations), which would form a loop whichever kind the anchor is, when the calendar is its own base, or when the resulting moments fall outside the timeline's range.
- **FR-005**: An anchor MUST, by default, **fix the moment**: when it is confirmed, the calendar's epoch is worked out and kept, and later edits to the base calendar don't move it. When defining the anchor, the owner MUST be able to choose instead to **keep the equation**: the calendar's epoch is worked out again whenever a calendar it depends on changes, so its anchor date always starts at the base date. A calendar that keeps an equation with its base **follows** it. The editor MUST show which kind each anchor is.
- **FR-005a**: For an anchor that fixes the moment, the editor MUST show the two dates as entered and, when the base calendar has since been edited so that the moment now falls at a different date in it, that date as well, so the difference is visible rather than silent.
- **FR-005b**: What an anchor that fixes the moment keeps is the moment its anchor date starts at. Editing the anchored calendar itself MUST keep its anchor date starting at that moment (its epoch is worked out again from it); if the edit removes the anchor date, the edit is refused with the reason, as any invalid anchor is (FR-004).
- **FR-006**: When an anchor that keeps the equation can no longer be resolved (its anchor date or base date no longer exists after an edit, or a calendar it follows is unusable), it MUST be listed as unusable with the reason and left out of the converter until fixed. Nothing is silently re-anchored.
- **FR-006a**: Deleting a calendar MUST NOT move any other calendar. The delete confirmation MUST name every calendar that follows it directly; confirming turns those anchors into anchors that fix the moment. Any anchor whose base was the deleted calendar MUST keep showing its dates as entered, noting the calendar is gone. If the reference calendar is deleted, every other calendar keeps its moment, and the first calendar added to an empty universe again becomes the reference calendar.
- **FR-007**: A new calendar MUST start, in the editor, anchored (fixing the moment) with the reference calendar as its base, or the first calendar in the list if there is none, and the first date of each as anchor date and base date, shown so the owner can change it before confirming.

**The converter**

- **FR-008**: The app MUST have a converter screen, available when a universe with at least one usable calendar is open, where the owner picks a calendar and enters a date as a value for each unit from the era's unit down.
- **FR-009**: The converter MUST show the moment where that date starts as a date in every other usable calendar of the universe, with each unit named, including the units above the era's unit.
- **FR-010**: When the moment doesn't fall on the start of the smallest unit in a target calendar, the converter MUST say so, at the precision chosen with a switch on the converter screen that applies to every result:
  - **partway** (the default): the unit it falls in, marked as partway through, with no amount;
  - **decimal**: the fraction of that unit to three decimal places, marked as rounded when it isn't exact;
  - **exact**: the exact fraction of that unit.
  It MUST never be shown in seconds. A moment that falls exactly on a unit's start is shown with no mark at every precision.
- **FR-010a**: The precision switch MUST keep its setting while the app is open and start at "partway" at each launch. It is not saved in the universe file.
- **FR-011**: A date the calendar refuses MUST be reported with the engine's reason at the value it concerns, and no conversion shown.
- **FR-012**: The converter MUST NOT show any absolute-timeline value or Earth unit.

**Storage**

- **FR-013**: Anchors MUST be stored in the project file as recorded in `/storage/schema.md` (updated by this feature's design PR), added by a numbered migration from schema version 2, and come back exactly as entered.
- **FR-014**: Migrating a version-2 file MUST keep every calendar at the moment it already had: the first calendar in the list becomes the reference calendar, and every other calendar is anchored to it, fixing the moment, with the first date of each as anchor date and base date.
- **FR-015**: Opening a file whose anchors can't be resolved (a loop, a missing base calendar, a date that doesn't exist) MUST still open the universe and report each affected calendar as in FR-006.

**Terms and layering**

- **FR-016**: Labels and messages MUST use the calendar model's terms (ADR 0009), and the Earth-terms check MUST keep passing.
- **FR-017**: Working out epochs from anchors MUST use exact arithmetic (principle III) and live below the app and the UI.

### Key Entities

- **Reference calendar**: the one calendar with no anchor, if there is one; its epoch is the fixed hidden point. A design and code term, not shown in the app.
- **Anchored calendar** and **base calendar**: the calendar an anchor places, and the calendar it's equated with. A calendar that keeps an equation with its base **follows** it.
- **Anchor**: a calendar's link into time: its base calendar, its anchor date (in the anchored calendar), its base date (in the base calendar), meaning the two dates started at the same moment when it was confirmed, and its kind: fix the moment (the default) or keep the equation.
- **Epoch (derived)**: where a calendar's first date starts on the hidden timeline, worked out from its anchor (kept, for an anchor that fixes the moment; worked out again through the chain of anchors, for one that keeps the equation); never shown or entered.
- **Conversion**: a date entered in one calendar, and the same moment as a date in each other calendar, with how far into the smallest unit it falls (kept exactly, shown at the converter's precision).

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
