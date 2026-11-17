# Feature Specification: Calendars in the Project File

**Feature Branch**: `design/007-calendars-in-project`

**Created**: 2026-11-16

**Status**: Draft

**Input**: User description: "Calendars in the project file (issue #52, Phase 1. Calendar core). The user can create, edit, delete and reopen their own calendars in the app. A minimal editor form defines a calendar with what the engine (uni-calendar, specs 004-006) already supports: its name, the fundamental unit's length, the single chain of groupings and subdivisions with their members' names and lengths, the era's unit, and adjustment rules typed as expressions (uni-expr, spec 005). The editor refuses an invalid calendar with the engine's reason. Calendars are saved in the project file (uni-store, spec 003; schema in /storage/schema.md) and come back unchanged when the project is reopened. This is the first time a calendar can be made and looked at in the app rather than only in tests. Agreed with the owner: this is the first of two specs for the 'first usable calendar' slice; spec 008 adds epoch anchoring (first calendar's epoch at a fixed hidden point on the timeline; later calendars anchor by equating one of their dates with a date in an existing calendar) and the converter screen. Out of scope here: epoch anchoring and the converter, formats, calendar grid, live preview and drift statistics, rule builder, multiple eras and pre-epoch labelling, parallel cycles, intercalary members, inserting or removing members, locales, presets, JSON import/export. Use the calendar model's own terms (ADR 0009)."

## Clarifications

### Session 2026-11-16

- Q: When the owner adds a new calendar, what does the editor form start with? → A: The smallest calendar the engine accepts: a fundamental unit and one era numbering it, with placeholder names; only the fundamental unit's length is left for the owner to enter. No groupings, subdivisions or rules, and nothing Earth-shaped.
- Q: Can an invalid calendar be kept as an unfinished draft? → A: No. Only calendars the engine accepts enter the universe; an invalid edit is fixed or discarded, and the last valid version stays. The file only ever holds valid calendars (an invalid one found on open is the edge case in FR-015).
- Q: How is the fundamental unit's length entered, given the UI never shows Earth units unless asked? → A: In SI seconds, clearly labelled as an SI physical input. SI is the physics' measuring stick, not part of any calendar: a field that takes SI says so, and SI is never presented as a calendar unit. The same rule will cover other physical inputs such as mass when bodies arrive.

**Issue**: #52 · **Depends on**: spec 003 (project file), spec 004 (calendar engine basics), spec 005 (expression language), spec 006 (adjustment rules) · **Followed by**: spec 008 (epoch anchoring and the converter) · **Phase**: 1. Calendar core · **Knowledge**: [/storage/schema.md](../../knowledge/storage/schema.md), [/calendar/calendar-model.md](../../knowledge/calendar/calendar-model.md), [/calendar/unit-kinds.md](../../knowledge/calendar/unit-kinds.md), [/calendar/adjustment-rules.md](../../knowledge/calendar/adjustment-rules.md), [/calendar/eras-and-epoch.md](../../knowledge/calendar/eras-and-epoch.md), [/language/expression-language.md](../../knowledge/language/expression-language.md), [/product/ui-views.md](../../knowledge/product/ui-views.md), [/time/display-units.md](../../knowledge/time/display-units.md), [/architecture/crate-layout.md](../../knowledge/architecture/crate-layout.md), [ADR 0002](../../knowledge/decisions/0002-hidden-absolute-timeline.md), [ADR 0008](../../knowledge/decisions/0008-physical-time-and-calendar-time.md), [ADR 0009](../../knowledge/decisions/0009-earth-neutral-names.md)

## User Scenarios & Testing *(mandatory)*

The examples use the Aster test calendar from specs 004 and 006: a sol of 88,775 seconds split into 20 bells of 100 pulses, an orbit of twelve spans counted in sols, and an adjustment rule that makes one span a sol longer in some orbits.

### User Story 1 - Define a calendar and keep it (Priority: P1)

With a universe open, the owner adds a calendar, fills in the editor form (its name, the fundamental unit and its length, the groupings above it with their members, the subdivisions below it, the era's name and unit, and any adjustment rules), and saves the calendar into the universe. They save the universe, close it, reopen the file, and the calendar is there exactly as they defined it.

**Why this priority**: It is the whole point of the feature: the first time a calendar exists in the app and survives in the project file. Everything else (editing, deleting, error reporting) builds on it.

**Independent Test**: Create a universe, define Aster through the form, save the file, reopen it, and compare every field of the reopened calendar with what was entered.

**Acceptance Scenarios**:

1. **Given** an open universe with no calendars, **When** the owner adds a calendar and enters Aster's definition and confirms it, **Then** Aster appears in the universe's list of calendars and the universe is marked as having unsaved changes.
2. **Given** a universe holding Aster, **When** the owner saves the universe, closes it and opens the file again, **Then** Aster is listed and its editor shows the same name, unit chain, member names and lengths, subdivisions, era and adjustment rules as were entered, with the rules' expressions exactly as typed.
3. **Given** a universe with several calendars, **When** it is saved and reopened, **Then** all of them come back, in the same order.
4. **Given** a universe file saved by an earlier version of the app (before calendars existed), **When** the owner opens it, **Then** it opens with no calendars, after the original is copied beside it as the existing migration rule requires.

---

### User Story 2 - Edit and delete calendars (Priority: P1)

The owner opens a saved calendar in the editor, changes it (renames a member, adds a subdivision, changes a rule's expression) and confirms the change, or abandons it. They can also delete a calendar they no longer want.

**Why this priority**: A calendar that can't be corrected is of little use, and the definer's first attempts will need fixing.

**Independent Test**: Edit each kind of field in turn, confirm, save and reopen, and check the change stuck; abandon an edit and check nothing changed; delete a calendar and check it is gone after save and reopen.

**Acceptance Scenarios**:

1. **Given** Aster in the universe, **When** the owner changes a member's length and confirms, **Then** the stored calendar has the new length and the universe is marked as having unsaved changes.
2. **Given** an edit in progress, **When** the owner abandons it, **Then** the calendar is exactly as it was before and nothing is marked as changed by the abandoned edit.
3. **Given** an edit in progress with changes, **When** the owner tries to leave the editor or close the universe, **Then** they are asked whether to keep or discard the edit first, before the existing unsaved-universe question.
4. **Given** Aster in the universe, **When** the owner deletes it and confirms the deletion, **Then** it is no longer listed, and after saving and reopening it is still gone.
5. **Given** Aster was deleted but the universe not yet saved, **When** the owner closes the universe without saving, **Then** the file still holds Aster.

---

### User Story 3 - Invalid calendars are refused with the reason (Priority: P1)

When the definition the owner enters can't become a calendar (a duplicate unit name, a member of zero length, an era's unit that doesn't exist, a rule expression that doesn't parse or reads the wrong variable, a fundamental unit length outside the engine's limits), the editor refuses to save it into the universe and shows the engine's reason next to the field it concerns, keeping everything the owner typed.

**Why this priority**: Principle VII: nothing invalid is silently accepted, wrapped or guessed. The engine already explains every refusal; this feature must show those explanations instead of hiding them.

**Independent Test**: For each kind of definition error the engine reports, enter a definition that triggers it, try to confirm, and check the calendar is not saved, the engine's message is shown at the right field, and the form still holds what was typed.

**Acceptance Scenarios**:

1. **Given** a definition with two units both named "sol", **When** the owner confirms, **Then** the calendar is not saved and the message names the duplicate name at the second unit.
2. **Given** an adjustment rule whose condition is `orbit %`, **When** the owner confirms, **Then** the calendar is not saved and the expression error (with its position) is shown under that rule's condition.
3. **Given** an adjustment rule naming a member that doesn't exist, **When** the owner confirms, **Then** the message names the rule and the missing member.
4. **Given** a refused definition, **When** the owner fixes the fault and confirms again, **Then** the calendar is saved.

---

### Edge Cases

- **No universe open**: adding or editing a calendar is unavailable until a universe is open.
- **Empty fields**: a missing name, member count, subdivision parts or fundamental length is refused with a message saying what is missing, never treated as zero or a default the owner didn't see.
- **Fundamental length forms**: the length can be a whole number or an exact decimal or fraction of seconds (for example `88775`, `88775.244` or `355/113`); it is stored and shown back exactly as the same value, never rounded.
- **Names that look alike**: unit names differing only in case ("Sol" and "sol") follow the engine's rule for uniqueness; the editor adds no rule of its own.
- **Two calendars with the same name**: refused within one universe, with a message naming the existing calendar.
- **Expressions with extra spaces or comments**: stored and shown back exactly as typed.
- **A file whose stored calendar the current engine refuses** (written by hand or by a future bug): the universe still opens; that calendar is listed as unusable with the engine's reason and can be opened in the editor to be fixed or deleted. Nothing is silently dropped.
- **Very large definitions**: a calendar with many members (for example a grouping of 1,000 members) can be entered, saved and reopened.
- **Earth's calendar words**: the app adds no Earth calendar words of its own to the editor (its labels use the model's terms: grouping, member, fundamental unit, subdivision, era's unit, adjustment rule); the owner's own names can be anything.

## Requirements *(mandatory)*

### Functional Requirements

**Calendars in a universe**

- **FR-001**: A universe MUST hold an ordered list of zero or more calendars, shown as a list when the universe is open.
- **FR-002**: The owner MUST be able to add a calendar, open any listed calendar in the editor, and delete a calendar after confirming.
- **FR-002a**: Adding a calendar MUST open the editor on the smallest calendar the engine accepts: a fundamental unit and one era numbering it, with placeholder names that use the model's terms, and the fundamental unit's length left empty for the owner to enter. It MUST have no groupings, subdivisions or adjustment rules.
- **FR-003**: Calendar names MUST be unique within a universe and follow the engine's rule for a calendar's name.
- **FR-004**: Adding, changing or deleting a calendar MUST mark the universe as having unsaved changes, and MUST reach the file only when the owner saves the universe (the document model of spec 003).

**The editor form**

- **FR-005**: The editor MUST let the owner enter everything a calendar definition holds today: the calendar's name; the fundamental unit's name, length in seconds and first number; the chain of groupings above it from top to bottom, each with its name, first number, member unit (name and first number) and ordered members (optional name and count); the chain of subdivisions below it, each with its name, parts and first number; the era's name and the unit it numbers; and the adjustment rules on the era's unit, each with its condition, the member it changes and the change.
- **FR-006**: The owner MUST be able to add, remove and reorder groupings, members, subdivisions and adjustment rules in the form.
- **FR-007**: Optional fields (first numbers, member names, a member unit when a grouping has a single member) MUST show the engine's default when left empty, and the stored calendar MUST keep them empty, not filled with the default.
- **FR-008**: The fundamental unit's length MUST be entered in seconds as a whole number, exact decimal or fraction, and kept exactly. The field MUST be labelled as an SI physical input (for example "Length, in SI seconds (physical)"), which `/time/display-units.md` allows: SI is the physics' measuring stick, never presented as a calendar unit. This is the only place the editor shows seconds, and no other Earth unit is shown.
- **FR-009**: The editor MUST NOT show the calendar's epoch or any absolute-timeline value. Every calendar's epoch is the fixed hidden point until spec 008 adds anchoring.

**Checking**

- **FR-010**: Confirming an edit MUST check the whole definition with the calendar engine. A definition the engine refuses MUST NOT be saved into the universe.
- **FR-011**: Every refusal MUST be shown with the engine's message, placed at the field, unit, member or rule it concerns, and the form MUST keep everything the owner typed.
- **FR-012**: Abandoning an edit MUST leave the stored calendar unchanged. Leaving the editor, closing the universe or closing the app with an edit in progress MUST ask whether to keep or discard the edit; keeping an invalid edit is refused as in FR-010, so the owner fixes it or discards it, and the last valid version stays. Unfinished drafts are never stored.

**Storage**

- **FR-013**: Calendars MUST be stored in the project file's tables as recorded in `/storage/schema.md` (updated by this feature's design PR), added by a numbered migration from schema version 1.
- **FR-014**: Opening a file MUST return every calendar exactly as saved: the same order, names, numbers, lengths, optional fields left empty, and adjustment rule expressions as their exact source text.
- **FR-015**: Opening a file whose stored calendar the engine refuses MUST still open the universe, list that calendar as unusable with the engine's reason, and let the owner fix or delete it.
- **FR-016**: Calendar storage MUST live in the storage component, which converts between stored rows and calendar definitions and depends on the calendar engine but never on the app or the UI. A new dependency edge between components is recorded in `/architecture/crate-layout.md` by the design PR.

**Terms**

- **FR-017**: The editor's labels and messages MUST use the calendar model's own terms (ADR 0009), and the Earth-terms check MUST keep passing.

### Key Entities

- **Calendar (stored)**: a calendar definition as the engine knows it (spec 004, `CalendarDef`), plus its position in the universe's list. Its epoch is not stored by this feature: every calendar's epoch is the fixed hidden point until spec 008.
- **Unit**: a grouping, member unit, the fundamental unit or a subdivision, with its name, kind, place in the chain and first number.
- **Member**: one member of a grouping, with its optional name, its count and its place.
- **Era**: the calendar's single era, with its name and the unit it numbers.
- **Adjustment rule**: a rule on the era's unit, with its condition's source text, the member it changes and the change.
- **Editor draft**: what the owner has typed but not yet confirmed; never stored in the file.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The owner can define the Aster test calendar, including its adjustment rule, through the form and save it into a universe in under 5 minutes.
- **SC-002**: 100% of calendars in a property test of at least 1,000 random valid definitions come back identical after being stored, saved to a file and reopened.
- **SC-003**: Every kind of definition error the engine reports is covered by a test that enters it through the form, and 100% of them are refused with the engine's message shown at the right field; none is saved.
- **SC-004**: Opening a universe with 20 calendars, including one with a 1,000-member grouping, shows its calendar list in under 1 second on the owner's machine.
- **SC-005**: A version-1 file (before calendars) opens and migrates with its backup copy made, and its universe name and other data are unchanged.
- **SC-006**: The Earth-terms, architecture and privacy checks pass.

## Assumptions

- **First of two specs**: this feature makes calendars exist in a project. Spec 008 adds epoch anchoring (the first calendar's epoch at the fixed hidden point; later calendars anchored by equating one of their dates with a date in an existing calendar) and the converter. Until then every calendar's epoch is the fixed hidden point, and there is no screen that turns dates into instants; the owner sees their calendar only as its definition.
- **What the engine supports today, nothing more**: one era, a single chain of groupings and subdivisions, a fixed fundamental unit on the absolute timeline's clock, adjustment rules on the era's unit only. The editor offers exactly that. Multiple eras, pre-epoch labelling, parallel cycles, intercalary members, inserting or removing members, event-delimited units, other clocks, locales and formats come with their own features, and the stored layout leaves room for them as `/storage/schema.md` already plans.
- **Plain form, no live preview**: the rule builder, calendar grid, live preview and drift statistics of the full calendar editor (`/product/ui-views.md`) are later features. Rules are typed as expressions.
- **Checked on confirm**: the definition is checked as a whole when the owner confirms it. Checking while typing can come later; it isn't needed for correctness.
- **Expressions stored as source text**: `/storage/schema.md` plans both source text and AST JSON for expressions. Whether the AST is stored now, or only the source text (which the engine re-parses on open), is a planning decision; the source text is always kept exactly.
- **Physical time and calendar time** (ADR 0008): the fundamental unit's length is the definer's choice, never checked against physics.
- **Knowledge update**: the design PR updates `/storage/schema.md` (the calendar tables actually built, schema version 2), `/time/display-units.md` (SI inputs are labelled as SI physical inputs and never presented as calendar units), `/architecture/crate-layout.md` (storage depending on the calendar engine, if the plan needs it) and `/product/ui-views.md` (the minimal editor as the first step towards the calendar editor), for the owner to re-verify.
- **Scope check (principle VIII)**: if planning shows the build PR would exceed about 30 minutes of review, the spec is split again (for example, storage first, then the editor) before the design PR is opened.
