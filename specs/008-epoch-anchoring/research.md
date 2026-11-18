# Research: Epoch Anchoring and the Converter

## Terms (agreed in design review)

- **Anchored calendar**: the calendar an anchor places. Its date in the anchor is the **anchor date** (`date`).
- **Base calendar**: the calendar it's equated with (`base`). Its date in the anchor is the **base date** (`base_date`).
- A calendar **follows** its base when its anchor keeps the equation, and follows every calendar its base follows. The calendars that follow a calendar directly are its **followers**.
- **Reference calendar**: the one calendar with no anchor, if there is one, at the hidden zero. A design and code term; the app only shows that a calendar isn't anchored.

## R1. Where a date starts, and putting a date at a moment

**Decision**: `uni-calendar` gains one method, `Calendar::epoch_for(&self, date: &Date, at: Instant) -> Result<Instant, CalendarError>`: the epoch that would make `date` start at `at`. It is `at − (start of date − this calendar's epoch)`, where the start is `date_to_instant(date).instant`, the first whole nanosecond at or after the exact start (spec 004, FR-014).

Because the epoch is a whole nanosecond, `ceil(epoch + d) = epoch + ceil(d)` for any exact offset `d`. So a calendar rebuilt with that epoch has `date_to_instant(date).instant == at` exactly, whatever its unit lengths. An anchor therefore makes the two dates start at **the same instant** on the timeline, and that holds in tests without any tolerance.

**Rationale**: the engine already turns dates into instants and back; this is the one missing piece, and it belongs with that arithmetic (principle III, exact; FR-017, below the app). The epoch stays an `Instant`, as spec 004 has it, so nothing else in the engine changes.

**Alternatives considered**: an exact rational epoch (sub-nanosecond). That would change `Instant` handling throughout the engine to fix a rounding the engine already reports (`rounded_by`). The spec accepts nanosecond rounding as long as it is reported (edge case "moments between whole nanoseconds").

## R2. What an anchor stores

**Decision**: an anchor is stored as:

- `kind`: `fixed` (fix the moment, the default) or `equation` (keep the equation), FR-005;
- `base`: the base calendar's id, or none once that calendar is deleted;
- `base_name`: the base calendar's name, written only when it is deleted, so the editor can say "Its base calendar, Aster, has since been deleted." (FR-006a);
- `date` and `base_date`: the two dates as entered, each a list of whole numbers from the era's unit down;
- `at`: for a fixed anchor, the instant both dates started at when the anchor was confirmed (whole seconds and nanoseconds from the hidden zero). It is none for an equation anchor, whose instant is worked out each time.

The epoch itself is **never stored**. It's worked out from the anchor every time the universe's calendars are placed (R4):

- reference calendar: the hidden zero (`Instant::ZERO`, as spec 007);
- fixed anchor: `epoch_for(date, at)` on the calendar's own current definition;
- equation anchor: `epoch_for(date, base.date_to_instant(base_date).instant)`, with the base calendar placed first.

**Rationale**:

- Keeping `at` rather than the epoch is what FR-005b asks for: an anchor that fixes the moment keeps its anchor date at that moment when the anchored calendar itself is edited.
- Keeping both dates as entered is FR-002 and FR-005a: the anchor is always shown as dates, and a fixed anchor can show its dates even after the base calendar changes or goes.
- `at` is a value on the hidden timeline. It lives in the file only; no command returns it and no screen shows it (principle IV). The constitution allows such values as technical data, as `Instant` in the engine already is.

**Alternatives considered**: storing the epoch (it would move the anchor date whenever the anchored calendar's own definition changes, against FR-005b); storing only the dates for fixed anchors (they would then follow the base calendar's edits, which is the equation kind).

## R3. Tables (schema version 3)

**Decision**: one migration, from version 2 to 3, adds one table. A calendar with no row is the reference calendar.

```
anchor(calendar_id INTEGER PRIMARY KEY REFERENCES calendar (id) ON DELETE CASCADE,
       kind TEXT NOT NULL CHECK (kind IN ('fixed', 'equation')),
       base_calendar_id INTEGER REFERENCES calendar (id) ON DELETE SET NULL,
       base_name TEXT,              -- set only when the base calendar has been deleted
       date TEXT NOT NULL,          -- JSON array of whole numbers, e.g. [1] or [412,3,9]
       base_date TEXT NOT NULL,
       at_secs INTEGER, at_nanos INTEGER,
       CHECK (base_calendar_id IS NULL OR base_calendar_id != calendar_id),
       CHECK ((at_secs IS NULL) = (at_nanos IS NULL)),
       CHECK (at_nanos IS NULL OR at_nanos BETWEEN 0 AND 999999999),
       CHECK (kind = 'fixed' OR base_calendar_id IS NOT NULL))
```

The migration then anchors every calendar after the first in the list (by `seq`) to the first one, fixed, with both dates `[1]` and `at` = (0, 0). Date `[1]` is the first instance of the era's unit, which starts at the epoch (spec 004: the era's unit is numbered by count from the epoch, and the one starting there is 1). In a version-2 file every epoch is the hidden zero, so every calendar stays where it was (FR-014, SC-004).

At most one calendar has no anchor row; once the reference calendar is anchored (FR-001a) or deleted, none does. That's checked in code when reading (more than one reads as a damaged file), because SQLite can't express "at most one row is missing" as a constraint.

**Rationale**:

- A table of its own, rather than columns added to `calendar`, can carry all its CHECKs. SQLite's `ALTER TABLE ADD COLUMN` can't add checks across columns, and rebuilding `calendar` would mean rebuilding the tables that refer to it.
- `ON DELETE SET NULL` on `base_calendar_id` and `ON DELETE CASCADE` on `calendar_id` keep the rows right if code forgets. `delete_calendar` still does the work FR-006a needs before it deletes (R6).
- Dates as JSON arrays: a date is a short list of numbers that's always read and written whole, and `/storage/schema.md` already plans `epoch_anchor_json`. One row per value would add a table for no gain. The format is strict, `[` whole numbers separated by `,` `]` with no spaces, written and read by a few lines in `uni-store`, so the crate needs no JSON library; anything else reads as a damaged file.

**Alternatives considered**: `epoch_anchor_json` on `calendar` holding the whole anchor (no foreign key to keep `base` right on delete); a `reference_calendar_id` on `universe` (two places saying the same thing).

## R4. Placing the universe's calendars

**Decision**: `uni-store` gains `OpenUniverse::placements() -> Vec<Placement>`, one per calendar in list order. Each holds the stored calendar and either its placed engine `Calendar` (with its epoch worked out) or a `PlacementProblem`. The work:

1. Build every calendar's definition (spec 007's `to_def` and `Calendar::new`). A refused definition is that calendar's problem, as in spec 007.
2. Place the reference calendar and every fixed anchor directly (R2).
3. Place equation anchors by following `base` with a depth-first walk. A calendar met again on the current path is a loop; every calendar on it gets the loop as its problem, naming the calendars in order. A calendar whose base calendar can't be placed gets "it's anchored to <base>, which can't be placed".

`PlacementProblem` covers: the definition refused; the anchor date or the base date refused by its calendar (the engine's `CalendarError`, as text); the base calendar unusable; a loop; the instant out of range; and a fixed anchor with no moment (only after R6's fallback). Each displays as a sentence in the model's terms.

The app calls `placements()` wherever it needs placed calendars: the list (problems), the editor (R7) and the converter (R9). It's cheap: placing is one `date_to_instant` per anchor, and the 20-calendar universe of SC-005 is listed in 15 ms today (spec 007, T021).

**Rationale**: one place decides where every calendar is, so the list, the editor and the converter can't disagree (FR-006, FR-015). Placing on demand rather than caching avoids a cache that edits must keep right.

**Alternatives considered**: storing a resolved epoch per calendar and updating dependants on each edit (more writes, and one missed update would silently misplace a calendar).

## R5. Checking an anchor on confirm (FR-004, FR-005, FR-005b)

**Decision**: `put_calendar` checks, after spec 007's steps 1–4 (completeness, name, length, engine):

5. the anchor's completeness: a new calendar in an empty universe, or the reference calendar, may have none (it may also be given one, FR-001a); any other calendar must have one; the base calendar exists and isn't the anchored calendar; and both dates have at least one value with no gap (a missing value followed by a given one);
6. the base calendar is usable (placed, R4); otherwise "Aster can't be used: <its problem>";
7. no loop, for either kind: the base calendar must not follow the anchored calendar. For an equation anchor, placing would lead back to the anchored calendar. For a fixed anchor, the instant would be read from a base that moves as soon as the anchored calendar does, so its two dates wouldn't match even as confirmed. The message names the chain: "Wick follows Tallow, so Tallow can't be anchored to it.";
8. the instant: for a fixed anchor whose kind, `base`, `date` and `base_date` are all unchanged from what's stored, the stored `at`; otherwise the start of `base_date` in the placed base calendar (its `CalendarError` placed at that date's value). An anchor whose base calendar has been deleted can be kept unchanged, but changing it needs a new base ("Choose a base calendar.", at `base`);
9. the anchor date at that instant: `epoch_for(date, at)` on the new definition (its `CalendarError` placed at the anchor date's value).

So editing a calendar's definition keeps a fixed anchor's moment (FR-005b), while changing anything in the anchor itself is a new equation, worked out from the base calendar as it is now: what the owner sees in the editor.

A refusal is a `FieldError` at the anchor (R8), and nothing is stored.

**Rationale**: FR-004 lists exactly these refusals. Steps 8 and 9 implement "fix the moment" as the clarification put it: confirmed once, then kept, including through edits to the calendar itself (FR-005b).

## R6. Deleting a calendar (FR-006a)

**Decision**: `OpenUniverse::delete_calendar(id)` now, in one transaction:

1. places the universe (R4), before the delete;
2. for every follower of `id` (an equation anchor with `id` as its base): sets its kind to `fixed` and `at` to the instant its anchor date starts at now. If that calendar couldn't be placed, `at` stays none and it reads as unusable ("its anchor has no moment: anchor it again"); this can only happen for a calendar that was already unusable;
3. for every anchor whose `base` is `id`, of either kind: writes `base_name` (the deleted calendar's name);
4. deletes the calendar (the cascade removes its own anchor; `SET NULL` clears `base`).

A new command, `calendar_followers(id) → string[]`, lists the followers of `id`, so the UI's confirmation can name them before anything is deleted.

Deleting the reference calendar leaves no reference calendar. The others keep their moments: every anchor with it as base was fixed or has just been fixed. From then on, a new calendar is anchored to the first calendar in the list (FR-007); only a calendar added to an empty universe becomes the reference again (FR-006a).

**Rationale**: the owner's answer to the second clarification. Fixing at the instant each follower has just before the delete means no calendar moves, which a test checks by converting the same dates before and after.

## R7. The editor (FR-001a, FR-002, FR-005, FR-005a, FR-007)

**Decision**: the editor gains a **Place in time** section, after the calendar's name:

- For a calendar with no anchor (the reference calendar, a term the app doesn't show): "Not anchored." and, when the universe has another calendar that can be placed, an **Anchor to a base calendar** button that opens the inputs below, starting as for a new calendar (FR-001a). No explanation of the hidden zero.
- Otherwise, or once that button is pressed:
  - **Base calendar**: a choice from a list, never typed (FR-002a). It holds every other calendar in list order, from `anchor_bases(id)` (R9). A calendar that can't be the base is shown greyed out and can't be chosen, with its reason after its name: "follows this calendar" (it follows the anchored calendar, R5 step 7) or "can't be placed" (R4). This is the same check `put_calendar` makes, so a choosable base is never refused as a loop on keep; the check on keep stays, for a universe that changed in between;
  - **Anchor date** and **Base date**: one whole-number field per unit from the era's unit down, labelled with the unit's name. Smaller units may be left empty (they take their first value, as the engine does). The anchor date's units come from the draft as it's being edited; the base date's come from the core (R9's `converter_calendars`). Each date is checked as it's typed (FR-002b) with `read_date` (R9): the base date against the stored base calendar, the anchor date against the draft's definition. A refused value is flagged at its field with the engine's message, as the converter does. If the draft's definition isn't valid yet, the anchor date shows "Checked once the calendar's definition is valid." instead;
  - **Read-back**: under the dates, a sentence built from the two `read_date` results: "Tallow's cycle 1, round 1, watch 1 began at the start of Aster's orbit 412, span 3 (Third), sol 9." Member names come in brackets after the number when the member has one. Values left empty are filled in with the unit's first value in the sentence, so it says exactly what the anchor means. There's no sentence while either date has a fault;
  - **Kind**: two choices, "Fix the moment: stays put if <base> is edited" (the default) and "Keep the equation: follows <base>".
  - There's no way to remove an anchor once kept (FR-002); discarding the edit is the way back before keeping.
  - When the base date starts between two whole nanoseconds (its `rounded_by`), the sentence "<Aster>'s date starts between two of the timeline's finest steps; this calendar is placed from the next one." (`CalendarDetails.rounded`, the spec's edge case on moments between whole nanoseconds). The epoch itself is always a whole nanosecond (R1), so this is the only rounding an anchor has.
  - For a fixed anchor, a note when it applies (FR-005a): "<Aster> has changed since: this moment is now <date> in it." or "Its base calendar, <Aster>, has since been deleted." The note is worked out by the core (`get_calendar`), from the stored `at`, and shown at the converter's default precision, "partway".

A new calendar starts anchored, fixing the moment, with the reference calendar as its base (or the first calendar in the list when there's none) and both dates `[1]`. A calendar added to an empty universe has no anchor and is the reference calendar (FR-007).

Anchor values are part of the draft, so the keep-or-discard guard of spec 007 covers them unchanged.

**Rationale**: dates are entered value by value because formats come later (spec assumption). The unit names make the fields self-explanatory and use the owner's own words.

## R8. Where anchor refusals are shown

**Decision**: `Location` gains `{ kind: "anchor"; field?: "base" | "kind" | "date" | "baseDate"; index?: number }`. `index` is the position of the value within a date, found from the unit a `CalendarError::InvalidValue` names. Errors that concern a whole date (`OutOfRange`) or the anchor as a whole have no `index`, or no `field`. The mapping is a total `match` over `CalendarError` and the anchor's own refusals, as spec 007 did for `DefinitionError`.

**Rationale**: FR-011's rule from spec 007, applied to the new section.

## R9. The converter

**Decision**: two commands.

- `anchor_bases(id | null) → AnchorBase[]`: every calendar except `id`, in list order, each with `ineligible: null | "follows" | "unplaceable"` (FR-002a). The UI words them "follows this calendar" and "can't be placed".
- `read_date(source, date) → DateReading | DateError`, where `source` is `{ id }` (a stored calendar, placed or not) or `{ form }` (an editor draft). It checks the date as `convert` does and returns its values with unit names and member names, filled in to the smallest unit. For a draft whose definition is refused, it returns `{ definition: <message> }` instead (FR-002b).
- `converter_calendars() → ConverterCalendar[]`: every calendar in list order with its id, name, its units from the era's unit down (the fields to fill in), and its problem if it can't be placed.
- `convert(from: id, date: (number | null)[]) → Conversion | DateError`:
  - The date is checked like an anchor's date (no gap; at least one value). The source calendar's `date_to_instant` gives the instant; a refusal is a `DateError { message, index }` at the value it concerns (FR-011).
  - For every other placed calendar, `instant_base_date(instant)` gives the date.
  - The result lists each target's units **above** the era's unit (Aster's age and cohort) and its date values, each with its unit's name and, for a member, its name if it has one (FR-009, as in the anchor's read-back), and how far into the smallest unit the moment falls (R10).
  - `Conversion` also lists every unusable calendar with its problem, so the screen can say why it isn't shown (FR-006), and `rounded: true` when the source date's exact start lay between two whole nanoseconds (the engine's `rounded_by`), so the screen can say so (edge case). The rounding is never given as an amount in seconds.

The screen: **Converter** is a button beside **Add calendar** on the calendar list, and the converter has **Back to calendars**. It holds a choice of calendar, the date's fields, the precision switch and the results. It converts as the owner types, whenever every filled field holds a whole number, so there's no button to press. A field holding anything else shows "must be a whole number" without calling the core, as in the editor. With one calendar, the results say "There's no other calendar to convert into." (Story 2, scenario 6). The converter has no unsaved state, so it doesn't take part in the keep-or-discard guard.

**Rationale**: FR-008–FR-012. Converting on each change avoids a button for what's a pure lookup. The core does all the arithmetic and gives the UI only text and whole numbers, so nothing is computed with JavaScript's floating-point numbers (principle III).

## R10. Partway through the smallest unit (FR-010, FR-010a)

**Decision**: `uni-calendar` gains `Calendar::smallest_unit_length() -> Ratio`: the fundamental unit's length divided by every subdivision's parts, in seconds, exact. For each target, the core works out `fraction = remainder / smallest_unit_length` (exact, `0 ≤ fraction < 1`) and returns:

```
partway: null                                  -- the moment is exactly at the unit's start
       | { decimal: "0.370", decimalExact: bool, exact: "37/100" }
```

`uni-time` gains `Ratio::to_decimal(places: u32) -> (String, bool)`, which truncates towards zero to `places` digits (so a moment in pulse 41 never shows as 42.000) and says whether that was exact. The converter asks for 3 places.

The precision switch is UI state held by the app's top component, so it keeps its setting while the app is open and resets to "partway" at launch (FR-010a). The UI renders a result from the precision chosen:

- partway: `pulse 41, partway through`;
- decimal: `pulse 41.370`, or `pulse ≈ 41.370` when `decimalExact` is false;
- exact: `pulse 41 + 37/100`.

The editor's FR-005a note uses the partway form.

**Rationale**: the owner's third and fourth answers. Giving the UI all three forms keeps the switch instant and keeps the arithmetic in the exact crates.

## R11. Tests (principle VI)

**Decision**:

- `uni-time`: `Ratio::to_decimal`: exact and inexact values, zero, values needing more places than asked, truncation (never rounding up).
- `uni-calendar`:
  - `epoch_for`: Aster and Tallow examples, dates before the epoch, a calendar with an awkward length (`355/113`) where the start is rounded, and out-of-range;
  - a property test (1,000 cases) on random calendars from the engine's existing generators: for a random date `X`, instant `t`, `Calendar::new` with `epoch_for(X, t)` gives `date_to_instant(X).instant == t`;
  - `smallest_unit_length`.
- `uni-store`:
  - the migration (a version-2 file with three calendars gains the table; the second and third are fixed anchors to the first at `[1]`; every calendar converts the same dates to the same instants as before; the `.v2.universe` copy is made);
  - anchor round trips (fixed and equation, `base_name`, `at`);
  - `placements()`: the reference, fixed, equation, chains, a loop (every member reported), an unusable base, a base date that no longer exists after an edit (fixed: moved date shown; equation: problem), and the former reference calendar anchored to a later calendar (FR-001a);
  - FR-005b (editing the anchored calendar keeps its date at `at`);
  - `delete_calendar` per R6 (followers fixed, nothing moves, `base_name` written, deleting the reference);
  - a property test (1,000 cases) of random chains of anchors: every anchor's two dates start at the same instant, and converting a random date from one calendar to another and back (through the instant) gives the date (SC-002).
- `uni-app`: `put_calendar` with each anchor refusal and its `Location` (SC-003); `new_calendar_form`'s anchor; `get_calendar`'s note; `calendar_followers`; `converter_calendars`; `convert` (the anchor example from Story 2, scenario 1, a refused value at its index, the partway forms, unusable calendars listed, one calendar only).
- UI (Vitest): the Place in time section for an unanchored calendar (and anchoring it) and an anchored one; changing the base calendar, the dates and the kind; the FR-005a notes; anchor field errors; the delete confirmation naming followers; the converter: choosing a calendar, entering a date, results with unit names, the precision switch and its reset only at launch, refusals at the field, the "no other calendar" message, unusable calendars listed.
- A timing check for SC-005 (20 calendars, one with 1,000 members, all anchored in a chain; one conversion), run by hand in release and recorded in `tasks.md`.

## R12. Size (principle VIII)

**Estimate**: about 3,300 changed lines.

- Rust code, about 1,000: `uni-calendar` and `uni-time` 120, store tables, anchors and placing 380, commands, locations, conversion, base list and date reading 500.
- Rust tests, about 1,050.
- UI code and tests, about 1,250: the Place in time section 350 (with the base list, live checks and read-back), the converter 300, tests 600.

That's more than spec 007's build (about 2,600, reviewed in one sitting) and within specs 004–006's (3,000–6,000). Tasks put anchoring (Stories 1 and 3) before the converter (Story 2), so if review shows it's too much, the converter can become its own build PR. No split now.

## R13. Knowledge changes

This design PR updates the following, each going back to draft for the owner to re-verify:

- `/storage/schema.md`: schema version 3 is built, with the `anchor` table as in R3; the planned `epoch_anchor_json` on `calendar` is replaced by it.
- `/calendar/eras-and-epoch.md`: the terms (anchored and base calendars, anchor and base dates, following); the reference calendar and anchoring it later; anchors by calendar dates of two kinds, fixing the moment (the default) or keeping the equation; what deleting a calendar does to anchors.
- `/product/ui-views.md`: the converter's first step: a date in one calendar, shown in every other, with the precision switch; observers' clocks come with physics.

It also adds a `log.md` entry. `/architecture/crate-layout.md` doesn't change: no new crate or dependency edge.
