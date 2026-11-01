---
description: "Task list for spec 001, app skeleton"
---

# Tasks: App Skeleton

**Input**: Design documents from `specs/001-app-skeleton/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Included. Constitution principle VI requires test-first work, and FR-014 requires the checks themselves to be tested. Test tasks come before the code they cover and must fail first.

**Organization**: by user story. Tasks marked **(owner)** are done by `origin-foundry`. Claude ticks them only after the owner confirms in conversation.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: US1 to US4 from spec.md

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: the owner's machine can build the app, and the repository can check that it can.

- [x] T001 Write `docs/development.md`. Sections: prerequisites (Rust at the version pinned in `rust-toolchain.toml` with the MSVC target; Microsoft C++ Build Tools with the "Desktop development with C++" workload; the WebView2 runtime; Node.js 22+; Python 3.11+), step-by-step installs for Windows, the two commands (`npm run dev`, `npm run app`), and "Certificate errors" (`NODE_OPTIONS=--use-system-ca` for Node/npm, and rustup's system-TLS setting, per research.md R3). Use the anchor names from contracts/check-scripts.md (`#c-build-tools`, `#nodejs`, `#certificate-errors`).
- [x] T002 [P] Write failing tests in `scripts/tests/test_check_prereqs.py` with a stubbed command runner. Cases: all present → exit 0 and `ok` lines; Rust missing → `missing  Rust … - see docs/development.md#rust`; Node 20 → `too old  Node.js 20.x (need 22 or later)`; on a non-Windows platform the build tools and WebView2 are reported as skipped; a Node HTTPS certificate error → a `warning` line. The script must never call an installer.
- [x] T003 Implement `scripts/check_prereqs.py` per contracts/check-scripts.md (standard library only; exit 0/1; find the build tools via `vswhere.exe`, WebView2 via its registry key) until T002 passes.
- [x] T004 **(owner)** Install the prerequisites on your Windows machine by following `docs/development.md`. Note how long it took; there's no target, because download sizes and speeds vary, especially for the C++ Build Tools. Then run `python scripts/check_prereqs.py`: every line should say `ok`. Report the time and any step that was unclear. (SC-001's 15-minute target is timed separately in T019, from a fresh checkout with prerequisites installed.)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: an empty workspace and UI toolchain that build cleanly. No user story can start before this.

- [x] T005 Create `rust-toolchain.toml` pinning the current stable Rust version exactly (e.g. `channel = "1.xx.x"`), with components `rustfmt` and `clippy`. Record the version in `docs/development.md`.
- [x] T006 Create the root `Cargo.toml` workspace: `members = ["crates/*"]`, `resolver = "3"`; `[workspace.package]` with `version = "0.1.0"`, `edition = "2024"`, `license = "GPL-3.0-only"`, `repository = "https://github.com/origin-foundry/universe"`; `[workspace.lints.rust] unsafe_code = "forbid"`; `[workspace.lints.clippy] all = "warn"`.
- [x] T007 [P] Create nine placeholder library crates: `crates/uni-time`, `crates/rebound-sys`, `crates/uni-expr`, `crates/uni-zones`, `crates/uni-calendar`, `crates/uni-format`, `crates/uni-physics`, `crates/uni-store`, `crates/uni-presets`. Each has a `Cargo.toml` (`version.workspace = true`, `edition.workspace = true`, `license.workspace = true`, `[lints] workspace = true`, **no dependencies**) and a `src/lib.rs` containing only a `//!` doc comment that gives the crate's purpose from `knowledge/architecture/crate-layout.md`.
- [x] T008 [P] In the four exact crates (`crates/uni-calendar/src/lib.rs`, `crates/uni-expr/src/lib.rs`, `crates/uni-format/src/lib.rs`, `crates/uni-zones/src/lib.rs`), add `#![deny(clippy::float_arithmetic)]` with a comment pointing to constitution principle III.
- [x] T009 [P] Create `deny.toml`: advisories (deny vulnerabilities and unmaintained crates), bans (`multiple-versions = "warn"`), sources (crates.io only; deny unknown registries and git sources). Leave licences unchecked, with a comment pointing to ADR 0004.
- [x] T010 Create the root `package.json` (`"private": true`, `"type": "module"`) with dependencies `react`, `react-dom`, `@tauri-apps/api` and dev dependencies `@tauri-apps/cli`, `vite`, `@vitejs/plugin-react`, `typescript`, `@types/react`, `@types/react-dom`, `eslint`, `@eslint/js`, `typescript-eslint`, `eslint-plugin-react-hooks`, `prettier`, `vitest`, `jsdom`, `@testing-library/react`, `@testing-library/jest-dom`. Scripts: `typecheck` (`tsc --noEmit`), `lint` (`eslint .`), `format:check` (`prettier --check .`), `test` (`vitest run`). Add `vite.config.ts` (`root: "ui"`, React plugin, Vitest with the `jsdom` environment), `tsconfig.json` (strict), `eslint.config.js` (flat config) and `.prettierrc.json`. Run `npm install` to create `package-lock.json`, using `NODE_OPTIONS=--use-system-ca` on the owner's machine.
- [x] T011 [P] Add `/crates/uni-app/gen/` to `.gitignore`, and add `.prettierignore` covering lock files, `target/`, `ui/dist/` and `crates/uni-app/gen/`.

**Checkpoint**: `cargo build --workspace`, `cargo clippy --workspace -- -D warnings`, `npm ci` and `npm run typecheck` all succeed.

---

## Phase 3: User Story 1 - Launch the empty app (Priority: P1) 🎯 MVP

**Goal**: a window titled "Universe" showing the name, the version and "No universe open", started by `npm run dev` or `npm run app`.

**Independent Test**: quickstart.md §1.

### Scaffold for User Story 1

- [x] T012 [US1] Create the Tauri app crate's scaffold, `crates/uni-app`, with **no `app_info` yet**:
  - `Cargo.toml`: workspace package fields and lints; dependencies `tauri` (2.x) and `serde` (with `derive`); build dependency `tauri-build`; **no workspace-crate dependencies yet**.
  - `build.rs` calling `tauri_build::build()`.
  - `src/main.rs`, which hides the console in release builds and calls `uni_app::run()`.
  - `src/lib.rs` with only `run()`, which builds and runs the Tauri app.
  - `tauri.conf.json`: `productName` "Universe"; identifier `io.github.origin-foundry.universe`; one window titled "Universe", 1200×800 and resizable; `devUrl` `http://localhost:5173`; `frontendDist` `../../ui/dist`; `beforeDevCommand` `npm run ui:dev`; `beforeBuildCommand` `npm run ui:build`; a strict CSP (`default-src 'self'`).
  - `capabilities/default.json` granting only `core:default`.
- [x] T013 [US1] Verify research.md R1's risk: from the repository root, `npx tauri info` finds `crates/uni-app/tauri.conf.json`. If it doesn't, try the CLI's option for naming the app directory. If neither works, **stop**: moving the crate needs a design PR that changes the crate layout. Record the outcome in the build PR description.

### Tests for User Story 1 (write first; they must fail)

- [x] T014 [P] [US1] In `crates/uni-app/src/lib.rs`, add a `#[cfg(test)]` unit test that `app_info()` returns `name == "Universe"` and `version == env!("CARGO_PKG_VERSION")` (contracts/app-info-command.md). Confirm it fails, because `app_info` doesn't exist yet.
- [x] T015 [P] [US1] Write `ui/src/App.test.tsx`, mocking `invoke` from `@tauri-apps/api/core`. (a) It resolves `{ name: "Universe", version: "9.9.9" }`: expect the texts "Universe", "9.9.9" and "No universe open". (b) It rejects: expect "Version unavailable" and "No universe open". Confirm it fails, because `App` doesn't exist yet.

### Implementation for User Story 1

- [x] T016 [US1] Create a placeholder app icon with a script (a plain 1024×1024 PNG of the letter "U", generated in code; no third-party artwork) and run `npx tauri icon` to fill `crates/uni-app/icons/`.
- [x] T017 [US1] In `crates/uni-app/src/lib.rs`, add `#[tauri::command] fn app_info() -> AppInfo { name, version }` and register it in `run()`. Create `ui/index.html`, `ui/src/main.tsx` and `ui/src/App.tsx`. `App` calls `invoke("app_info")` once on mount and shows the name, the version (or "Version unavailable" if the call fails) and "No universe open". Work until T014 and T015 pass.
- [x] T018 [US1] Add npm scripts to `package.json`: `ui:dev` (`vite`), `ui:build` (`vite build`), `tauri` (`tauri`), `dev` (`tauri dev`), `app` (`tauri build --no-bundle`, then launch `target/release/uni-app.exe`), and `predev`/`preapp` (`python scripts/check_prereqs.py`).
- [x] T019 [US1] **(owner)** Run quickstart.md §1 on your machine, timing from a fresh clone of the build branch to the first window (SC-001 target: under 15 minutes, with prerequisites already installed): `npm ci`; `npm run dev` (check the window's three texts; edit "No universe open" and see it update within 3 s); close the window cleanly; `npm run app`; then take Node off `PATH` temporarily and check that `npm run dev` stops with the prerequisites report. Report the timings (SC-001, SC-002).

**Checkpoint**: User Story 1 works end to end on the owner's machine.

---

## Phase 4: User Story 2 - Every change is checked automatically (Priority: P1)

**Goal**: every applicable CI job runs on every PR, and any failure blocks the merge.

**Independent Test**: quickstart.md §2.

- [x] T020 [US2] In `.github/workflows/ci.yml`, update the `detect` job: `rust` = `Cargo.toml` exists; `ui` = root `package.json` exists; `tauri` = `crates/uni-app/tauri.conf.json` exists.
- [x] T021 [US2] In `ci.yml`, make the `rust` job run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` on Windows, and add `--exclude uni-app` to the clippy and test commands on Ubuntu (research.md R6).
- [x] T022 [US2] In `ci.yml`, make the `ui` job run from the repository root: drop `working-directory: ui`, set `cache-dependency-path: package-lock.json`, and run `npm ci`, `typecheck`, `lint`, `format:check`, `test` and `npm audit --audit-level=high`.
- [x] T023 [US2] In `ci.yml`, make the `app-build (windows)` job run `npm ci` then `npm run tauri -- build --no-bundle` from the repository root.
- [x] T024 [US2] Check that `rust-deps` (cargo-deny) passes against the new `deny.toml` with the real `Cargo.lock`.
- [x] T025 [US2] Locally, introduce each deliberate failure from quickstart.md §2's first six rows one at a time (Rust formatting, a Clippy warning, a failing Rust test, a TypeScript type error, a failing UI test, a Rust syntax error in `uni-app`). Confirm that the same command CI runs fails with a message naming the problem, then revert each. Also prove the advisory checks (spec US2 scenario 5): temporarily add a dependency version with a published RustSec advisory to a placeholder crate, and confirm `cargo deny check advisories` fails naming the advisory; do the same for npm with a package version that has a published high-severity advisory and `npm audit --audit-level=high`. Revert both. Record all results in the build PR description.

**Checkpoint**: on the build PR, every job runs and passes, none is skipped, and they finish within 20 minutes (SC-003, SC-005).

---

## Phase 5: User Story 3 - The agreed architecture can't be broken by accident (Priority: P2)

**Goal**: upward dependencies, unlisted edges and floating point in exact crates are rejected, with rules read from the knowledge concept.

**Independent Test**: quickstart.md §3, plus the fixture tests.

### Tests for User Story 3 (write first; they must fail)

- [x] T026 [P] [US3] Create fixtures under `scripts/tests/fixtures/architecture/`, each a minimal Cargo workspace plus its own `layout.md` containing a `crate-dependencies` block (data-model.md):
  - `ok/`
  - `upward/` (a lower crate depends on a higher one)
  - `unlisted/` (an edge the block doesn't allow)
  - `missing-crate/` (a workspace crate absent from the block)
  - `absent-crate/` (a listed crate that doesn't exist)
  - `cycle/` (the block itself has a cycle)
  - `bad-block/` (malformed YAML)
  - `float-type/` (`f64` in an exact crate)
  - `float-literal/` (`1.0` in an exact crate)
  - `float-ignored/` (`f64` and `1.0` only in comments and string literals: must pass)
- [x] T027 [US3] Write `scripts/tests/test_check_architecture.py`: one test per fixture asserting the exit code (0/1/2) and the finding prefix and text from contracts/check-scripts.md (e.g. `error: dependency: upper -> lower`, `error: float: src/lib.rs:3: f64 in exact crate`). Skip with a clear message when `cargo` isn't installed, as on the `knowledge` job.

### Implementation for User Story 3

- [x] T028 [US3] Implement `scripts/check_architecture.py` per contracts/check-scripts.md and data-model.md:
  - Parse the single block opened with ```` ```yaml crate-dependencies ```` (the `exact` key lists the exact crates).
  - Validate it ("every name in a list is itself a key", "no crate depends on itself", "no cycles", "every crate named in `exact` is a key").
  - Compare it with `cargo metadata --format-version 1 --no-deps` (normal, build and dev dependencies on workspace crates).
  - Scan the exact crates' `.rs` files for `f32`/`f64` and float literals, ignoring comments and string literals.
  
  Work until T027 passes.
- [x] T029 [US3] Add an `architecture` job to `ci.yml` (Ubuntu; Rust toolchain; Python 3.13 with PyYAML). It runs `python -m unittest discover -s scripts/tests -p test_check_architecture.py -v` (`scripts/tests` isn't a Python package, so it's discovered by path), then `python scripts/check_architecture.py`, and runs whenever `detect` finds Rust.
- [x] T030 [US3] Run `python scripts/check_architecture.py` on the real workspace: it must pass. Then temporarily add `uni-calendar` as a dependency of `uni-physics` and `let x: f64 = 1.0;` to `uni-calendar`, confirm both are reported, and revert.

**Checkpoint**: the architecture rules hold, and breaking them is caught locally and in CI.

---

## Phase 6: User Story 4 - Dependencies stay up to date (Priority: P3)

**Goal**: Dependabot proposes grouped, policy-compliant Rust and npm updates.

**Independent Test**: quickstart.md §4.

- [ ] T031 [US4] In `.github/dependabot.yml`, add `cargo` (directory `/`) and `npm` (directory `/`) entries: weekly, each with one group matching `*`, labelled `fast-lane`, alongside the existing `github-actions` entry. Remove the comment about adding them later.

**Checkpoint**: the configuration covers all three ecosystems.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [ ] T032 [P] Add a "Development" section to `README.md` linking `docs/development.md`, and update its status line to say the app skeleton exists.
- [ ] T033 Run the full local verification and record the results in the build PR description:
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
  - `npm run typecheck`, `npm run lint`, `npm run format:check`, `npm test`
  - `python scripts/check_architecture.py`
  - `python -m unittest discover -s scripts/tests`
  - `python scripts/check_okf.py knowledge`
- [ ] T034 Open the build PR as the bot: label `build`, body with `Design: #<design PR>` and `Closes #9`. List generated files (lock files, icons) separately from hand-written files, so the review stays near 30 minutes (plan.md Complexity Tracking). Move card #9 to **PR review**.

---

## After merge (owner, outside the PR's task list)

These happen after the build PR is merged, so they aren't checkboxes. The PR policy requires every task above to be ticked, and these can't be done before the merge.

1. **Required checks** (FR-016). In Settings → Rules → Rulesets → `main` → **Require status checks to pass** → **Add checks**, add: `rust (ubuntu-latest)`, `rust (windows-latest)`, `rust-deps`, `ui`, `app-build (windows)` and `architecture`. Keep `knowledge` and `policy`. Save.
2. **Follow-up design PR** (Claude): add `scripts/check_architecture.py` to the PR policy's guardrail files (research.md R7).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: T001 → T002 → T003 → T004. T004 is the owner's hand-off, and nothing local can be built without it.
- **Foundational (Phase 2)**: needs T004. Blocks every user story.
- **US1 (Phase 3)**: needs Phase 2. T012 (scaffold) comes first; T013 must pass before T014 to T018.
- **US2 (Phase 4)**: needs Phase 2. T023 needs T012 and T018.
- **US3 (Phase 5)**: needs Phase 2 (a real workspace for T030). T026 to T028 can start any time after Phase 2.
- **US4 (Phase 6)**: independent; any time after T010 (`package.json` exists).
- **Polish (Phase 7)**: after all stories.

### Within each story

Scaffold, then tests, then implementation (T012 and T013, then T014 and T015, before T017; T026 and T027 before T028), then implementation, then verification.

### Parallel Opportunities

- T007, T008, T009 and T011 touch different files.
- T014 (Rust test) and T015 (UI test) are in different languages and files.
- US3's fixtures and tests (T026, T027) can be written while US1 is in progress.
- US4 (T031) can be done at any point after T010.

## Parallel Example: Phase 2

```text
Task: "T007 Create nine placeholder library crates under crates/"
Task: "T009 Create deny.toml"
Task: "T011 Update .gitignore and add .prettierignore"
```

## Implementation Strategy

### MVP first

1. Phase 1, including the owner's install (T004).
2. Phase 2.
3. Phase 3 (US1). **Stop and validate** with the owner (T019).

### Then

4. US2 and US3 (the checks), then US4, then Polish, in one build PR.

### Notes

- Every task is ticked in `tasks.md` as it's done. The PR policy refuses a build PR with unticked tasks.
- Build PRs may only change `tasks.md` within `specs/`. Anything learned during the build that changes the design (for example, T015 failing) needs a new design PR.
