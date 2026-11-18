# Implementation Plan: Epoch Anchoring and the Converter

**Branch**: `design/008-epoch-anchoring` (design), `build/008-epoch-anchoring` (build) | **Date**: 2026-11-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/008-epoch-anchoring/spec.md` (approved by the owner, 2026-11-18)

## Summary

Calendars in a universe get places in time. The first calendar is the **reference calendar**, silently at the hidden zero, as every calendar is today; it can be anchored later. Every other calendar has an **anchor** that places it, the **anchored calendar**, from a **base calendar**: its **anchor date** equated with a **base date**, either **fixing the moment** (the default: the instant is worked out on confirm and kept) or **keeping the equation** (worked out again from the base each time, so the anchored calendar **follows** its base). A calendar can't be anchored to one that follows it.

- `uni-calendar` gains `epoch_for`, which gives the epoch that puts a date at an instant exactly, to the nanosecond.
- `uni-store` stores anchors in a new `anchor` table (schema version 3) and places every calendar through one function, which reports loops, unusable calendars and dates that no longer exist.
- `uni-app` checks anchors on confirm, keeps followers in place when a calendar is deleted, and adds the converter's commands.
- The UI gains a **Place in time** section in the editor and a converter screen with a partway / decimal / exact switch.

No command or screen gives a timeline value. Details are in [research.md](research.md).

## Technical Context

**Language/Version**: Rust 1.99.1 (pinned), edition 2024; TypeScript with React (as spec 001).

**Primary Dependencies**: workspace crates `uni-time`, `uni-calendar`, `uni-store`, `uni-app`; `rusqlite` (as spec 003); `serde` in `uni-app` (as spec 007); `uni-store` writes and reads the anchors' short arrays of whole numbers itself, so it needs no JSON library (research R3); Tauri 2. Dev-only: `proptest` (already in `uni-calendar` and `uni-store`); Vitest and Testing Library in the UI. No new third-party dependency.

**Storage**: the `.universe` SQLite file, schema version 2 → 3 (research R3).

**Testing**: `cargo test` (unit, property and migration tests), Vitest for the UI, and a hand-run timing check (research R11).

**Target Platform**: Windows desktop (the app); Rust crates tested on Windows and Linux in CI.

**Project Type**: desktop app (Tauri) with core library crates.

**Performance Goals**: one conversion across 20 calendars, one with 1,000 members, in under 1 s (SC-005).

**Constraints**: exact arithmetic only in `uni-time` and `uni-calendar`, and no floating point in the UI's handling of dates or fractions; no timeline value outside the file; no Earth calendar words in code (the Earth-terms check).

**Scale/Scope**: about 3,000 changed lines (research R12).

## Constitution Check

*GATE: passed before research and again after design.*

| Principle | How this plan complies |
|---|---|
| I. Spec before code | Approved spec; this design PR, then a build PR. |
| II. Knowledge is the source of truth | Follows `/calendar/eras-and-epoch.md`, `/storage/schema.md`, `/product/ui-views.md`, `/time/display-units.md`, ADR 0002. This PR updates schema (version 3), eras and epoch (the reference calendar and the two anchor kinds) and UI views (the first converter), back to draft for the owner to re-verify, with a `log.md` entry. |
| III. Exact calendar arithmetic | `epoch_for`, `smallest_unit_length` and `Ratio::to_decimal` are exact. The core sends the UI only whole numbers and text, so the UI never computes a fraction (research R10). |
| IV. Hidden absolute timeline | A fixed anchor's instant is stored in the file but never returned by a command; anchors and results are dates only. Rounding is reported as a fact, not an amount (research R9). |
| V. Layering | No new crate or edge: `uni-store` → `uni-calendar`, `uni-time` and `uni-app` → all three exist since spec 007. No Tauri in core crates. Placement lives in `uni-store`, below the app (FR-017). |
| VI. Test-first | Each task writes its tests first: `to_decimal`, `epoch_for` with a 1,000-case property test, the migration, placements, a 1,000-case property test of anchor chains, every anchor refusal's location, the converter commands, UI tests. No presets. |
| VII. No silent ambiguity | Loops, unusable calendars, dates that no longer exist and nanosecond rounding are all reported; a fixed anchor whose base calendar changed shows where its moment now falls; deleting a calendar names the calendars it affects. |
| VIII. Small changes | About 3,000 lines, in the range of specs 004–007's builds, each reviewed in one sitting. Tasks put anchoring before the converter, so a split stays possible. |

## Project Structure

### Documentation (this feature)

```text
specs/008-epoch-anchoring/
├── spec.md, plan.md, research.md, data-model.md, quickstart.md
├── contracts/anchors-and-converter.md
├── checklists/requirements.md
└── tasks.md
```

### Source Code

```text
crates/uni-time/src/ratio.rs          Ratio::to_decimal
crates/uni-calendar/src/convert.rs    Calendar::epoch_for
crates/uni-calendar/src/calendar.rs   Calendar::smallest_unit_length
crates/uni-store/src/schema.rs        migration 2 → 3
crates/uni-store/src/anchor.rs        Anchor, AnchorKind, reading and writing anchor rows,
                                      placements(), check_anchor(), followers() (new)
crates/uni-store/src/calendar.rs      CalendarRecord.anchor; delete_calendar per R6
crates/uni-store/src/lib.rs           re-exports
crates/uni-app/src/calendars.rs       anchor in the form, CalendarDetails, anchor locations
crates/uni-app/src/converter.rs       converter_calendars, convert, ConvertedDate (new)
crates/uni-app/src/lib.rs             three new commands; get_calendar's new shape
ui/src/AnchorSection.tsx              Place in time (new)
ui/src/Converter.tsx                  the converter screen (new)
ui/src/convertedDate.ts               rendering a ConvertedDate at a precision (new)
ui/src/calendarApi.ts, calendarDraft.ts, CalendarEditor.tsx, CalendarList.tsx, App.tsx
ui/src/*.test.tsx                     UI tests
knowledge/storage/schema.md, knowledge/calendar/eras-and-epoch.md, knowledge/product/ui-views.md, knowledge/log.md
```

**Structure Decision**: the existing workspace. Engine arithmetic in `uni-calendar`, storage and placement in `uni-store`, translation for the UI in `uni-app`, screens in `ui/src`, as specs 003 and 007 set out.

## Complexity Tracking

No violations.
