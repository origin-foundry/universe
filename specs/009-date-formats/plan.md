# Implementation Plan: Date Formats

**Branch**: `design/009-date-formats` (design), `build/009-date-formats` (build) | **Date**: 2026-11-23 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/009-date-formats/spec.md` (approved by the owner, 2026-11-23)

## Summary

Calendars get **formats**: named patterns of literal text and fields (`{orbit}.{span:02}.{sol:02}`, `{sol} {span:name}, orbit {orbit} {era}`), the subset of `/formats/templates.md`'s syntax this spec needs. Each calendar has one **main format**, the owner's choice or a **generated** one that reads like today's text. A format that can't be read back unambiguously is **write-only**, with the reason.

- `uni-format` (empty until now) compiles patterns against a calendar, writes dates, and reads text back into every date it can mean, reporting where text stops fitting, values that disagree, and dates the calendar refuses.
- `uni-calendar` gains `member_names` and `fraction_through`, so a format can name members and the converter can say how far into the last unit a format shows a moment falls.
- `uni-store` stores formats in a new `format` table (schema version 4).
- `uni-app` checks formats live and on keep, and writes converter results and the anchor's read-back sentence in each calendar's main format.
- The UI gains a **Formats** section in the calendar editor. The converter keeps value-by-value input; nothing in the app reads text yet (clarifications 3 and 4).

Details are in [research.md](research.md).

## Technical Context

**Language/Version**: Rust 1.99.1 (pinned), edition 2024; TypeScript with React (as spec 001).

**Primary Dependencies**: workspace crates `uni-time`, `uni-calendar`, `uni-format`, `uni-store`, `uni-app`; `rusqlite` (as spec 003); `serde` in `uni-app`; Tauri 2. New edges, all already allowed by `/architecture/crate-layout.md`: `uni-format` → `uni-calendar`, `uni-time`; `uni-app` → `uni-format`. Dev-only: `proptest` (already in the workspace) in `uni-format`; Vitest and Testing Library. No new third-party dependency.

**Storage**: the `.universe` SQLite file, schema version 3 → 4 (research R7).

**Testing**: `cargo test` (unit, property and migration tests), Vitest, and a hand-run timing check (research R11).

**Target Platform**: Windows desktop (the app); Rust crates tested on Windows and Linux in CI.

**Project Type**: desktop app (Tauri) with core library crates.

**Performance Goals**: write and read one date in under 10 ms with a 1,000-member grouping (SC-005).

**Constraints**: exact arithmetic only in `uni-calendar` and `uni-format`; no timeline value or Earth unit on screen; no Earth calendar words in code (the Earth-terms check).

**Scale/Scope**: about 2,800 changed lines (research R12).

## Constitution Check

*GATE: passed before research and again after design.*

| Principle | How this plan complies |
|---|---|
| I. Spec before code | Approved spec; this design PR, then a build PR. |
| II. Knowledge is the source of truth | Follows `/formats/templates.md`, `/formats/parsing.md`, `/calendar/unit-kinds.md`, `/storage/schema.md`, `/product/ui-views.md`. This PR updates templates and parsing (what's built, write-only formats, the generated format), schema (version 4) and UI views (the formats section), back to draft for the owner to re-verify, with a `log.md` entry. |
| III. Exact calendar arithmetic | `uni-format` already denies float arithmetic; `fraction_through` works in the engine's exact ticks. The UI never computes a fraction (as spec 008). |
| IV. Hidden absolute timeline | Formats write calendar dates only; no command returns a timeline value. |
| V. Layering | No new crate; the two new edges are in the allowed list. No Tauri in core crates. Formats live in `uni-format`, below the app (FR-019). |
| VI. Test-first | Each task writes its tests first. Every format has a round-trip property test (SC-002), as the principle requires: the generated formats of the test calendars and a set of owner-style patterns, 1,000 cases each. No presets. |
| VII. No silent ambiguity | Text that can mean several dates gives every reading; disagreeing values, refused dates and where text stops fitting are reported; a format that can't be read back is marked write-only with the reason. |
| VIII. Small changes | About 2,800 lines, less than spec 008's build. US3 (converter and read-back) is its own phase, so it can move to a second build PR if the build runs long. |

## Project Structure

### Documentation (this feature)

```text
specs/009-date-formats/
├── spec.md, plan.md, research.md, data-model.md, quickstart.md
├── contracts/formats.md
├── checklists/requirements.md
└── tasks.md
```

### Source Code

```text
crates/uni-calendar/src/calendar.rs   Calendar::member_names
crates/uni-calendar/src/convert.rs    Calendar::fraction_through
crates/uni-format/Cargo.toml          uni-calendar, uni-time; proptest (dev)
crates/uni-format/src/lib.rs          re-exports, crate docs
crates/uni-format/src/pattern.rs      Format, Part, compile, generated, PatternError (new)
crates/uni-format/src/write.rs        writing (new)
crates/uni-format/src/read.rs         reading, ReadError (new)
crates/uni-format/src/fixtures.rs     test calendars; round_trip.rs, timing.rs (test modules, new)
crates/uni-store/src/schema.rs        migration 3 → 4
crates/uni-store/src/format.rs        FormatRecord rows (new)
crates/uni-store/src/calendar.rs      CalendarRecord.formats, main_format
crates/uni-app/Cargo.toml             uni-format
crates/uni-app/src/calendars.rs       formats in the form, check_format, Location::Format, read_date text
crates/uni-app/src/converter.rs       ConvertedDate text, generated, unit; partway in that unit
crates/uni-app/src/lib.rs             the check_format command
ui/src/FormatsSection.tsx             the editor's Formats section (new)
ui/src/calendarApi.ts, calendarDraft.ts, CalendarEditor.tsx, AnchorSection.tsx, convertedDate.ts, Converter.tsx
ui/src/*.test.tsx                     UI tests
knowledge/formats/templates.md, knowledge/formats/parsing.md, knowledge/storage/schema.md, knowledge/product/ui-views.md, knowledge/log.md
```

**Structure Decision**: the existing workspace. Formats in `uni-format`, unit arithmetic in `uni-calendar`, storage in `uni-store`, translation for the UI in `uni-app`, screens in `ui/src`.

## Complexity Tracking

No violations.

## For the owner to confirm in design review

- The generated format names a member unit's members only when all are named (research R3), so a partly named grouping shows numbers only, where today it shows the names it has.
- With an owner's main format, the converter shows exactly what the format says; units above the era's unit appear only if the format includes them (R3).
- The read-back sentence writes the full date, not just the values entered (R10).
