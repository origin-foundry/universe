# Implementation Plan: App Skeleton

**Branch**: `design/001-app-skeleton` (design), `build/001-app-skeleton` (build) | **Date**: 2026-11-01 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/001-app-skeleton/spec.md` (approved by the owner, 2026-11-01)

## Summary

Stand up the empty Universe desktop app and the checks that guard every later change:

- **The app:** a Cargo workspace with the ten agreed crates as placeholders, and a Tauri 2 app crate (`crates/uni-app`) with a React UI in `ui/`. One root `package.json` provides `npm run dev` (live reload) and `npm run app` (optimised build), both guarded by a prerequisites check.
- **The checks:** CI's existing Rust, UI and app-build jobs switch on, and a new `architecture` job enforces the downward-only crate dependencies and the float ban. It reads the allowed dependencies from a machine-readable block in `/architecture/crate-layout.md`.
- **Dependencies:** Dependabot covers Cargo and npm.

Details and alternatives are in [research.md](research.md).

## Technical Context

**Language/Version**: Rust stable, edition 2024, one exact version pinned in `rust-toolchain.toml` (R5). TypeScript in strict mode. Python 3.11+ for the repository scripts.

**Primary Dependencies**: Tauri 2 (`tauri`, `tauri-build`, `@tauri-apps/cli`, `@tauri-apps/api`); React; Vite; ESLint with typescript-eslint; Prettier; Vitest with Testing Library. PyYAML for the architecture check.

**Storage**: N/A (the project file is spec 003).

**Testing**: `cargo test` (one Rust unit test for `app_info`), Vitest (UI component tests), Python `unittest` with fixture workspaces for the architecture and prerequisite checks.

**Target Platform**: Windows 10 and 11 (x86-64, MSVC) for the app. Core crates are also built and tested on Ubuntu (R6).

**Project Type**: desktop app (Tauri) with a Rust library workspace.

**Performance Goals**: window shown within 5 s of launch after the first build; live reload within 3 s (SC-002).

**Constraints**:
- all PR checks finish within 20 min (SC-005), helped by Rust build caching;
- no installer or signing;
- least-privilege Tauri capabilities (R11);
- downloads must work behind the owner's TLS interception (R3).

**Scale/Scope**: about 10 placeholder crates and one UI component. The hand-written code is small, but generated lock files are large.

## Constitution Check

*GATE: checked before research and again after design. Both passed.*

| Principle | How this plan complies |
|---|---|
| I. Spec before code | Approved spec. This design PR holds the spec, plan, tasks and knowledge change; the build PR links to it. |
| II. Knowledge is the source of truth | The allowed dependencies are added to `/architecture/crate-layout.md`, which the check reads (FR-012a). That concept goes back to draft and needs the owner's re-verification. A `log.md` entry is added. |
| III. Exact calendar arithmetic | Enforced from the first commit: the architecture check's float scan, plus Clippy's `float_arithmetic` lint in the four exact crates (R5, R7). |
| IV. Hidden absolute timeline | Not affected; nothing about time is shown. |
| V. Layering | Enforced by the architecture check (R7). The edges are set explicitly (R8), and Tauri is only used in `uni-app`. |
| VI. Test-first | Fixture tests for the architecture check, a Rust test for `app_info` and UI component tests are written before the code they cover (task order). Deliberate-failure fixtures guard the checks themselves (FR-014). |
| VII. No silent ambiguity | The prerequisites check and the architecture check name every problem. The UI shows "Version unavailable" rather than a blank. |
| VIII. Small changes | See Complexity Tracking: the hand-written code fits a ~30-minute review, but the generated files make the diff look larger. |
| Human gates | The bot does the build. Adding required checks is handed to the owner (FR-016). The guardrail change (Stop hook) is in this design PR. |

## Project Structure

### Documentation (this feature)

```text
specs/001-app-skeleton/
├── spec.md
├── plan.md               # this file
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── app-info-command.md
│   └── check-scripts.md
├── checklists/requirements.md
└── tasks.md              # /speckit-tasks
```

### Source Code (repository root)

```text
Cargo.toml                # workspace: members = ["crates/*"], shared version, [workspace.lints]
rust-toolchain.toml       # pinned stable + rustfmt, clippy
deny.toml                 # cargo-deny: advisories, bans, sources
package.json              # UI + Tauri CLI deps; scripts: dev, app, typecheck, lint, format:check, test
package-lock.json
vite.config.ts            # root: ui/
tsconfig.json
eslint.config.js
.prettierrc.json

crates/
├── uni-time/             # placeholder lib (lib.rs + Cargo.toml)
├── rebound-sys/          # placeholder lib; C build arrives in Phase 3
├── uni-expr/             # exact crate: float lint on
├── uni-zones/            # exact crate
├── uni-calendar/         # exact crate
├── uni-format/           # exact crate
├── uni-physics/
├── uni-store/
├── uni-presets/
└── uni-app/              # Tauri app crate: main.rs, lib.rs (app_info), build.rs,
                          # tauri.conf.json, capabilities/default.json, icons/

ui/
├── index.html
└── src/
    ├── main.tsx
    ├── App.tsx           # name, version (via app_info), "No universe open"
    └── App.test.tsx

scripts/
├── check_architecture.py
├── check_prereqs.py
└── tests/
    ├── test_check_architecture.py
    ├── test_check_prereqs.py
    └── fixtures/architecture/   # tiny fake workspaces, one per rule

docs/development.md       # prerequisites, commands, certificate troubleshooting
.github/workflows/ci.yml  # detect updated; new 'architecture' job; Linux excludes uni-app
.github/dependabot.yml    # + cargo, npm (grouped, fast-lane)
```

**Changed in this design PR (not the build PR)**:

- `knowledge/architecture/crate-layout.md`: adds the `crate-dependencies` block, and notes the root `package.json` and the Tauri app crate in `crates/uni-app`. It goes back to draft until the owner re-verifies it.
- `knowledge/log.md`: an entry for that change.
- `scripts/hooks/stop_checks.py`: runs the architecture check when it exists (R14). Covered by an added hook test.

**Structure Decision**: a Cargo workspace with `crates/*` as specified by the verified crate layout, a single npm package at the root, and the UI's sources in `ui/` (R1).

## Complexity Tracking

| Concern | Why it's needed | Simpler alternative rejected because |
|---|---|---|
| The build PR's diff exceeds a 30-minute review once `Cargo.lock`, `package-lock.json` and the generated icons are counted | Lock files make builds reproducible, and Tauri needs icons to build | Leaving lock files out makes CI non-deterministic. Instead the PR description lists the generated files separately, so the review covers only hand-written files (about 35 small files). |
| Two Python scripts plus fixtures alongside the app | FR-004a and FR-012 to FR-014 need them, and the checks must be tested | Shell scripts wouldn't run the same on Windows and Linux. |
