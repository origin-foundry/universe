# Implementation Plan: Calendars in the Project File

**Branch**: `design/007-calendars-in-project` (design), `build/007-calendars-in-project` (build) | **Date**: 2026-11-17 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/007-calendars-in-project/spec.md` (approved by the owner, 2026-11-17)

## Summary

The project file gains calendars. `uni-store` stores a `CalendarRecord`: the engine's definition as the owner typed it, with the fundamental unit's length kept as exact text. It's stored in four new tables (schema version 2), converted to a `CalendarDef` through one path that the editor and the file share. `uni-time` gains an exact `Ratio::parse`. `uni-app` adds five commands that check a form with the engine and place each refusal at the field it concerns. The UI shows the open universe's calendar list and a plain editor form, with a keep-or-discard guard in front of the existing file flows. Every calendar's epoch is the fixed hidden point until spec 008. Details are in [research.md](research.md).

## Technical Context

**Language/Version**: Rust 1.99.1 (pinned), edition 2024; TypeScript with React (as spec 001).

**Primary Dependencies**: workspace crates `uni-time`, `uni-calendar`, `uni-store`, `uni-app`; `rusqlite` (as spec 003); `serde` in `uni-app`; Tauri 2. Dev-only: `proptest` in `uni-store`; Vitest and Testing Library in the UI. No new third-party dependency.

**Storage**: the `.universe` SQLite file, schema version 1 → 2 (research R2).

**Testing**: `cargo test` (unit, round-trip, property and migration tests), Vitest for the UI, and a hand-run timing check (research R11).

**Target Platform**: Windows desktop (the app); Rust crates tested on Windows and Linux in CI.

**Project Type**: desktop app (Tauri) with core library crates.

**Performance Goals**: listing a universe of 20 calendars, one with 1,000 members, in under 1 s (SC-004).

**Constraints**: no floating point in `uni-time` or `uni-calendar` (exact crates); the length is parsed exactly; the epoch never shown; no Earth calendar words in code (the Earth-terms check).

**Scale/Scope**: about 2,600 changed lines (research R12).

## Constitution Check

*GATE: passed before research and again after design.*

| Principle | How this plan complies |
|---|---|
| I. Spec before code | Approved spec; this design PR, then a build PR. |
| II. Knowledge is the source of truth | Follows `/storage/schema.md`, `/calendar/*`, `/time/display-units.md`, `/product/ui-views.md`. This PR updates schema (version 2 as built), display units (SI inputs labelled as such) and UI views (the plain form first), back to draft for the owner to re-verify, with a `log.md` entry. |
| III. Exact calendar arithmetic | The length goes from text to `Ratio` exactly in `uni-time`; the UI never turns it into a JavaScript number. `uni-calendar` is unchanged. |
| IV. Hidden absolute timeline | The epoch is `Instant::ZERO`, set in code, never stored or shown (FR-009). Seconds appear only on the length field, labelled as an SI physical input (clarification 3, `/time/display-units.md`). |
| V. Layering | New edges `uni-store` → `uni-calendar`, `uni-time` are already allowed by `/architecture/crate-layout.md`; `uni-app` already depends on both. No Tauri in core crates. |
| VI. Test-first | Tests are written before the code they cover: `Ratio::parse`, migration, record round trips and a 1,000-case property test, an error-location test per engine error, command tests, UI tests. No presets. |
| VII. No silent ambiguity | Every refusal shows the engine's message at its field; missing values are reported, never defaulted; a stored calendar the engine refuses is listed with its reason, not dropped. |
| VIII. Small changes | About 2,600 lines, below specs 004–006's builds (3,000–6,000), each reviewed in one sitting. Tasks put storage before the editor, so a split stays possible. |

## Project Structure

### Documentation (this feature)

```text
specs/007-calendars-in-project/
├── spec.md, plan.md, research.md, data-model.md, quickstart.md
├── contracts/calendar-commands.md
├── checklists/requirements.md
└── tasks.md
```

### Source Code

```text
crates/uni-time/src/ratio.rs        Ratio::parse
crates/uni-time/src/error.rs        TimeError::NotANumber
crates/uni-store/Cargo.toml         + uni-calendar, uni-time, publish = false; dev proptest
crates/uni-store/src/schema.rs      migration 1 → 2; foreign keys on
crates/uni-store/src/calendar.rs    CalendarRecord, to_def, reading and writing rows (new)
crates/uni-store/src/universe.rs    calendars(), add/replace/delete_calendar
crates/uni-store/src/error.rs       CalendarName, CalendarNameTaken, NoSuchCalendar
crates/uni-store/tests/ or src/*    round-trip, property, migration, timing tests
crates/uni-app/src/calendars.rs     CalendarForm, FieldError, locations, starting form (new)
crates/uni-app/src/lib.rs           five commands registered
ui/src/CalendarList.tsx             list and Add calendar (new)
ui/src/CalendarEditor.tsx           the form (new)
ui/src/calendarApi.ts               command wrappers and types (new)
ui/src/fileFlows.ts, App.tsx        keep-or-discard guard; list/editor in the main area
ui/src/*.test.tsx                   UI tests
knowledge/storage/schema.md, knowledge/time/display-units.md, knowledge/product/ui-views.md, knowledge/log.md
```

**Structure Decision**: the existing workspace. Storage logic stays in `uni-store`, translation for the UI in `uni-app`, and the screens in `ui/src`, as spec 003 set out.

## Complexity Tracking

No violations.
