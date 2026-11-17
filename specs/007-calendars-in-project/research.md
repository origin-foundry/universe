# Research: Calendars in the Project File

## R1. One stored shape, used by the editor and the file

**Decision**: `uni-store` gains a `CalendarRecord`: a calendar exactly as the owner entered it, with the same fields as spec 004's `CalendarDef` except two. The fundamental unit's length is the **text** the owner typed (`"88775.244"`), and there's no epoch. `CalendarRecord::to_def()` parses the length (R3), sets the epoch to the fixed hidden point (R6) and returns a `CalendarDef`. `uni_calendar::Calendar::new` then checks it. The editor's confirm and the file's open both go through this one path, so a calendar can't pass the editor and then fail on open, or the other way round.

**Rationale**: FR-014 asks for everything back exactly as entered, including empty optional fields. `CalendarDef` already keeps optional fields as `Option` (spec 004), so only the length needs its text kept. One conversion path means the editor and the file check calendars the same way (FR-010, FR-015).

**Alternatives considered**: storing `CalendarDef` itself, with the length as a numerator and denominator. That loses what the owner typed (`88775.244` would come back as `22193811/250`), and SQLite has no 128-bit integers for the parts. A JSON blob per calendar was also considered and rejected: the schema design (`/storage/schema.md`) uses real tables, so later features (spec 008's anchors, formats, locales) can refer to a calendar's units and eras by row.

## R2. Tables (schema version 2)

**Decision**: one migration, from version 1 to 2, adds four tables. Each has only the columns this feature uses. A calendar's id stays stable across edits, because spec 008's anchors will refer to it.

```
calendar(id INTEGER PRIMARY KEY, seq INTEGER NOT NULL UNIQUE, name TEXT NOT NULL UNIQUE)
unit(id INTEGER PRIMARY KEY, calendar_id → calendar ON DELETE CASCADE, seq INTEGER NOT NULL,
     kind TEXT NOT NULL CHECK (kind IN ('grouping','fundamental','subdivision')),
     name TEXT NOT NULL, first_number INTEGER,
     member_unit_name TEXT, member_unit_first_number INTEGER,   -- groupings only
     length TEXT,                                                -- fundamental only
     parts INTEGER,                                              -- subdivisions only
     UNIQUE (calendar_id, seq))
unit_member(id INTEGER PRIMARY KEY, unit_id → unit ON DELETE CASCADE, seq INTEGER NOT NULL,
            name TEXT, count INTEGER NOT NULL, UNIQUE (unit_id, seq))
rule(id INTEGER PRIMARY KEY, unit_id → unit ON DELETE CASCADE, seq INTEGER NOT NULL,
     condition_src TEXT NOT NULL, member TEXT NOT NULL, change INTEGER NOT NULL,
     UNIQUE (unit_id, seq))
era(id INTEGER PRIMARY KEY, calendar_id → calendar ON DELETE CASCADE, seq INTEGER NOT NULL,
    name TEXT NOT NULL, unit_name TEXT NOT NULL, UNIQUE (calendar_id, seq))
```

`unit.seq` orders the single chain from top to bottom (groupings, the fundamental unit, then subdivisions). A grouping's member unit is a pair of columns on the grouping's row, as in `CalendarDef` (`MemberUnitDef`), not a row of its own. CHECK constraints keep each kind's columns where they belong. Saving an edited calendar replaces its units, members, rules and era in one transaction and keeps the `calendar` row. Foreign keys are switched on (`PRAGMA foreign_keys = ON`) on every connection, so deleting a calendar deletes the rest of it.

**Rationale**: these are the tables `/storage/schema.md` plans, cut to what exists today. Real columns rather than `params_json` let SQLite check each kind's fields. The schema design changes in four ways, all recorded in the concept by this design PR:
- `unit` gets `seq` and real columns instead of `parent_id` and `params_json`. A single chain needs only an order; `parent_id` comes with branching trees.
- `plural` waits for formats.
- the `calendar` table's clock, epoch and pre-epoch columns wait for spec 008 and multiple eras. The clock is always the absolute timeline today.
- `rule` stores the member and change as columns, not `modification_json`, because a change is always "lengthen or shorten one member" today.

**Alternatives considered**: `parent_id` now (it needs a separate rule for member units and gives nothing a single chain needs); `ON DELETE` handled in code (easy to forget in later features).

## R3. Entering the length exactly

**Decision**: `uni-time` gains `Ratio::parse(&str)`. It accepts an optional sign, then a whole number (`88775`), a decimal (`88775.244`) or a fraction (`355/113`), with no spaces inside, no exponent and no thousands separators. The result is exact or an error (`TimeError::NotANumber`, `DivisionByZero` or `Overflow`). The text the owner typed is what's stored (R1), and it's shown back as typed. Leading and trailing spaces are trimmed before the length is parsed and stored. The engine still checks the value (it must be positive and within its limits, spec 004 research R3).

**Rationale**: exact arithmetic (principle III) all the way from the keyboard. `uni-time` already owns `Ratio`, and it's an exact crate, so the parser can't use floating point. Keeping the text means the owner sees `88775.244` again, not a fraction.

**Alternatives considered**: parsing in the UI (JavaScript numbers are floating point, so it isn't exact); parsing in `uni-app` (the store also needs it on open).

## R4. Names, and calendar names in a universe

**Decision**: the editor trims leading and trailing spaces from every name before checking, so `" sol"` and `"sol"` can't look like two different units. Otherwise names are kept exactly, and the engine's rules decide uniqueness within a calendar (exact match, so `Sol` and `sol` differ). Calendar names must be non-empty, at most 200 characters (as for universe names, spec 003) and unique within the universe by exact match after trimming. `uni-store` checks uniqueness and the `UNIQUE` constraint backs it up.

**Rationale**: FR-003 and the "names that look alike" edge case: the editor adds no case rule of its own. Trimming matches spec 003's universe names. The 200-character limit stops pathological input and matches the one name limit the app already has.

## R5. Commands between the UI and the core

**Decision**: `uni-app` gains five Tauri commands (contract in [contracts/calendar-commands.md](contracts/calendar-commands.md)):

- `list_calendars() → CalendarSummary[]`: id, name, and `problem` (the engine's message when a stored calendar is refused, FR-015).
- `get_calendar(id) → CalendarForm`: the form contents for one calendar.
- `new_calendar_form() → CalendarForm`: the starting form (FR-002a, R7).
- `put_calendar(id | null, form) → { state, calendar } | FieldError`: checks the form (R8) and adds or replaces the calendar. The universe becomes dirty.
- `delete_calendar(id) → state`.

`CalendarForm` is a JSON mirror of `CalendarRecord`. Whole-number fields are `number | null`. The UI turns any text that isn't a whole number into a field error before sending, and the core reports `null` in a required field as missing. Returning the universe state after every change keeps the window title's unsaved-changes mark current, as spec 003's commands do.

**Rationale**: the store and the engine stay free of serde and Tauri (principle V); `uni-app` is the only translator, as in spec 003.

## R6. The epoch until spec 008

**Decision**: every calendar's epoch is `Instant::ZERO`, set in `CalendarRecord::to_def()`, not stored and never shown (FR-009). Spec 008 adds the anchor columns and the first-calendar rule.

**Rationale**: the owner chose this in conversation: the first calendar's epoch sits at the fixed hidden point, and later calendars anchor to existing ones in spec 008.

## R7. The starting form (FR-002a)

**Decision**: `new_calendar_form()` returns a calendar named "Calendar" (or "Calendar 2", "Calendar 3", … if taken), a fundamental unit named "unit" with an empty length and empty first number, and one era named "Era" numbering "unit". It has no groupings, subdivisions or rules. Confirming it without a length is refused ("the fundamental unit's length is missing").

**Rationale**: it's the smallest calendar the engine accepts once a length is entered. The placeholder names are the model's own neutral terms (ADR 0009), and it's served by the core so the UI doesn't need its own copy of the rules.

## R8. Checking and placing errors (FR-010, FR-011)

**Decision**: `put_calendar` checks in this order, stopping at the first fault:

1. form completeness in `uni-app`: missing required numbers or length;
2. the calendar's name: empty, too long, or already used;
3. the length's text (R3);
4. `Calendar::new` (every `DefinitionError` from specs 004 and 006).

A fault comes back as a `FieldError { message, at }`. `message` is the engine's text (its `Display`), unchanged. `at` is a location the UI understands: `calendar-name`, `era`, `unit(i)` with an optional field, `member(i, j)`, or `rule(i, k)` with an optional field. `uni-app` maps each `DefinitionError` variant to a location. For variants that name a unit or member, the location is the matching index in the form; for a duplicate name, it's the second occurrence. The mapping is a total `match`, so a new engine error can't compile without a location, and a unit test covers every variant.

**Rationale**: the engine reports one fault at a time with names, not positions. Mapping names back to positions in `uni-app` keeps the engine unaware of forms. One fault at a time matches the engine; the owner fixes it and confirms again (Story 3, scenario 4).

**Alternatives considered**: collecting every fault at once (the engine stops at the first by design); placing errors in the UI by matching message text (fragile).

## R9. Opening a file with a calendar the engine refuses (FR-015)

**Decision**: on open, every stored calendar goes through `to_def()` and `Calendar::new`. A failure doesn't stop the open. The calendar is listed with `problem` set to the message, and `get_calendar` still returns its form, so the owner can fix or delete it. The rows themselves must still be readable. A file whose rows break the tables' own constraints is reported as damaged, as spec 003 does today.

**Rationale**: principle VII: nothing silently dropped. A bad calendar shouldn't lock the owner out of the rest of the universe.

## R10. The editor screen and pending edits (FR-012)

**Decision**: with a universe open, the main area shows the calendar list with **Add calendar**. Choosing a calendar, or adding one, replaces the list with the editor. The editor has **Keep changes** (`put_calendar`), **Discard changes** and, for a stored calendar, **Delete calendar** (which asks first). The form is a set of sections:
- the calendar's name;
- the groupings, top to bottom, each with its members and its rules if it's the era's unit;
- the fundamental unit;
- the subdivisions;
- the era.

Each list has add, remove, move up and move down buttons. While the draft differs from what's stored, File → New, Open, Close, Exit and the window's close button first ask "Keep or discard your changes to <calendar>?" (keep runs `put_calendar` and stops if it's refused). Then the existing unsaved-universe question follows. The guard sits in front of the existing flows in `fileFlows.ts`.

**Rationale**: the plain form the spec asks for, with no new screen frameworks or routing. Asking about the draft first keeps the two questions separate.

## R11. Tests (principle VI)

**Decision**:
- `uni-time`: tests for `Ratio::parse` (each accepted form, each refusal, exactness, overflow).
- `uni-store`:
  - a migration test (a version-1 file gains the tables and keeps its name; a copy is made, as spec 003 already tests);
  - round trips for hand-made records (Aster with its rule, every optional field empty and set, a 1,000-member grouping);
  - a property test of 1,000 random valid records: store, save to a file, reopen, compare (SC-002);
  - delete and cascade tests;
  - a test that a hand-broken stored calendar opens with its problem.
- `uni-app`: a test per `DefinitionError` variant for its location (SC-003), and command tests through the session like spec 003's.
- UI (Vitest): the list, add, edit, discard, delete, field errors shown at the right input, and the keep-or-discard guard on Close.
- A timing check for SC-004 (20 calendars, one with 1,000 members, listed in under 1 s), run by hand in release and recorded in `quickstart.md`.

The random-record generator lives in `uni-store`'s tests: the engine's own generators aren't exported, and exporting them would be a test-only API.

## R12. Size (principle VIII)

**Estimate**: about 2,600 changed lines.
- Rust code, about 750: `Ratio::parse` 80, store tables and records 300, commands and error locations 370.
- Rust tests, about 900.
- UI code and tests, about 950: the editor 550, tests 400.

That's less than specs 004–006's builds, which the owner reviewed in one sitting each. Tasks are grouped so storage (Story 1's core) lands before the editor, which would allow a split if review shows it's needed. No split now.

## R13. Knowledge changes

This design PR updates the following, each going back to draft for the owner to re-verify:
- `/storage/schema.md`: schema version 2 is built; the calendar tables as in R2 replace the planned rows for `calendar`, `unit`, `unit_member`, `rule` and `era`; expressions are stored as source text for now, with AST JSON when something needs it.
- `/time/display-units.md`: SI inputs are labelled as SI physical inputs and never presented as calendar units (clarification 3).
- `/product/ui-views.md`: the calendar editor's first step is a plain form.

It also adds a `log.md` entry. `/architecture/crate-layout.md` doesn't change: `uni-store` may already depend on `uni-calendar` and `uni-time`.
