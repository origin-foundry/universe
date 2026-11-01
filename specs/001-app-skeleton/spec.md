# Feature Specification: App Skeleton

**Feature Branch**: `design/001-app-skeleton`

**Created**: 2026-11-01

**Status**: Approved by the owner, 2026-11-01

**Input**: User description: "App skeleton (issue #9, Phase 0 spec 001). The Universe desktop app opens a window on Windows, and every automated check runs on every PR: Rust format/lint/tests on Windows and Linux, dependency advisories, UI type/lint/format/tests, a Windows app build, an architecture-rules check (crate dependencies only point downward per knowledge/architecture/crate-layout.md; no floating point in calendar, expression, format or zone crates), and Dependabot updates for Rust and npm dependencies. The workspace contains the agreed crates (knowledge/architecture/crate-layout.md) as empty placeholders. Out of scope: time types (spec 002), the project file (spec 003), any calendar or physics behaviour."

**Issue**: #9 · **Phase**: 0. Foundations · **Knowledge**: [/architecture/stack.md](../../knowledge/architecture/stack.md), [/architecture/crate-layout.md](../../knowledge/architecture/crate-layout.md), [/decisions/0007-bot-identity-and-gates.md](../../knowledge/decisions/0007-bot-identity-and-gates.md)

## Clarifications

### Session 2026-11-01

- Q: Where should the architecture check read the allowed component dependencies from? → A: From a machine-readable list inside the knowledge concept `/architecture/crate-layout.md`; it's the single source of truth, so changing it needs a design PR and the owner's re-verification.
- Q: What should the launch command start: the development version or a built app? → A: Both, as two documented commands: one starts the app in development mode with live reload, the other builds the optimised app and runs it.
- Q: How should the prerequisites get onto a machine? → A: Installed natively on Windows by the owner, following step-by-step documentation, plus a script that checks what's installed and reports exactly what's missing; nothing installs itself. Docker was considered and rejected: containers can't run the Windows app's window, and CI already provides clean, reproducible builds.

## User Scenarios & Testing *(mandatory)*

The people involved are the **owner** (who runs the app and approves every change), **Claude** (who proposes changes as the bot account) and **Dependabot** (which proposes dependency updates).

### User Story 1 - Launch the empty app (Priority: P1)

The owner starts Universe on their Windows machine and sees its main window. There is nothing to do in it yet, but it shows what it is and confirms that no universe is open.

**Why this priority**: Every later feature is something the owner sees and uses in this window. Until it opens, nothing else in Phase 0 can be demonstrated.

**Independent Test**: On a Windows machine with the documented prerequisites, follow the setup instructions from a fresh checkout and launch the app.

**Acceptance Scenarios**:

1. **Given** a fresh checkout and the documented prerequisites, **When** the owner runs the documented development command, **Then** a window titled "Universe" opens, and editing the window's text updates it without restarting.
2. **Given** a fresh checkout and the documented prerequisites, **When** the owner runs the documented run command, **Then** the optimised app is built and a window titled "Universe" opens.
3. **Given** the app is open, **When** the owner looks at the window, **Then** it shows the app's name, its version, and a message that no universe is open.
4. **Given** the app is open, **When** the owner closes the window, **Then** the app exits cleanly with no error output.

---

### User Story 2 - Every change is checked automatically (Priority: P1)

When Claude opens a pull request, every automated check that applies to the code in the repository runs on it, and a failure in any of them blocks the merge. The owner is only handed a PR once all of them pass.

**Why this priority**: The approval gates from ADR 0007 depend on these checks existing and running. Without them, "all checks passed" means very little.

**Independent Test**: Open a PR that deliberately breaks one rule at a time (formatting, a lint warning, a failing test, a UI type error, a broken app build) and confirm each one fails its check and blocks the merge, then fix it and confirm the PR is handed to the owner.

**Acceptance Scenarios**:

1. **Given** a PR whose code is correctly formatted, lint-clean and passing its tests, **When** checks finish, **Then** every check passes on both Windows and Linux where it applies, and the ready gate assigns the owner.
2. **Given** a PR that introduces a formatting error, a lint warning or a failing test in the core code, **When** checks run, **Then** the corresponding check fails and the PR can't be merged.
3. **Given** a PR that introduces a type error, lint error, formatting error or failing test in the user interface, **When** checks run, **Then** the user-interface check fails.
4. **Given** a PR that breaks the Windows app build, **When** checks run, **Then** the app-build check fails.
5. **Given** a PR that adds a dependency with a known security advisory, **When** checks run, **Then** the dependency check fails.

---

### User Story 3 - The agreed architecture can't be broken by accident (Priority: P2)

The core is split into the components agreed in the knowledge bundle. A change that makes a lower-level component depend on a higher-level one, or that introduces floating-point numbers into the components that must stay exact, is rejected automatically.

**Why this priority**: These rules (constitution principles III and V) are easiest to enforce from the very first line of code; retrofitting them later is expensive. They matter less than having a running app and running checks, so P2.

**Independent Test**: Open PRs that each break one rule (an upward dependency, a new dependency edge not in the agreed layout, a floating-point type in an exact component) and confirm each is rejected with a message naming the rule and the offending component.

**Acceptance Scenarios**:

1. **Given** the agreed component layout, **When** checks run on an unchanged repository, **Then** the architecture check passes.
2. **Given** a PR that makes a component depend on one above it in the layout (for example, physics depending on calendars), **When** checks run, **Then** the architecture check fails and names both components.
3. **Given** a PR that uses a floating-point type in the calendar, expression, format or zone components, **When** checks run, **Then** the architecture check fails and names the file and line.
4. **Given** a PR that adds a dependency between components that the agreed layout doesn't list, **When** checks run, **Then** the architecture check fails and says the layout must be changed in a design PR first.

---

### User Story 4 - Dependencies stay up to date (Priority: P3)

Dependabot proposes updates to the core's and the user interface's third-party dependencies on a regular schedule, grouped so they don't conflict, in a form the PR policy accepts.

**Why this priority**: Useful hygiene, but nothing is broken without it for a while. GitHub Actions updates already work (PR #8).

**Independent Test**: After merge, confirm that Dependabot's configuration covers the core and user-interface dependency manifests, and that its next scheduled run opens grouped PRs that pass the PR policy.

**Acceptance Scenarios**:

1. **Given** the merged configuration, **When** Dependabot next runs, **Then** it opens at most one PR for core dependencies and one for user-interface dependencies, each labelled `fast-lane`.
2. **Given** such a PR, **When** checks run, **Then** the PR policy accepts it, because it only changes dependency manifests and lock files.

### Edge Cases

- **Prerequisites missing**: the prerequisites check reports each missing or too-old tool by name, with its required version and the documentation step that installs it. The development and run commands run the same check first and stop with that report, rather than an opaque build error.
- **Network that intercepts encrypted connections** (as on the owner's machine): the setup instructions say how to make dependency downloads trust the system certificate store.
- **Checks that don't apply yet**: a check whose code area doesn't exist is reported as skipped, not failed, and doesn't block the hand-over to the owner.
- **A check runs on only one platform**: platform-specific failures (Windows only or Linux only) are reported per platform, so the owner can see which one failed.
- **Empty components**: placeholder components with no behaviour still build, lint and test cleanly, so the checks are meaningful from the first PR.

## Requirements *(mandatory)*

### Functional Requirements

**The app**

- **FR-001**: The app MUST open a single main window titled "Universe" on Windows.
- **FR-002**: The window MUST show the app's name, its version and a "No universe open" message.
- **FR-003**: Closing the window MUST exit the app cleanly.
- **FR-004**: The repository MUST document the prerequisites and two commands that work from a fresh checkout on Windows: a **development** command that launches the app with live reload of both the user interface and the core, and a **run** command that builds the optimised app and launches it.
- **FR-004a**: The repository MUST provide a prerequisites check that reports, for each required tool, whether it's installed and at a supported version, and links each missing one to its documented install step. It MUST NOT install anything.

**The core components**

- **FR-005**: The core MUST be split into exactly the components listed in the knowledge bundle's crate layout, each present as a placeholder with no behaviour.
- **FR-006**: The dependencies between components MUST match the crate layout's downward-only rule. Placeholders MUST only declare dependencies the layout allows.
- **FR-007**: The app MUST be able to call into the core (one placeholder call is enough to prove the connection, for example the version shown in the window).

**Automated checks** (all run on every PR to `main` and on every push to `main`)

- **FR-008**: Core code MUST be checked for formatting, for lint warnings (treated as errors) and by its test suite, on both Windows and Linux.
- **FR-009**: Core dependencies MUST be checked against known security advisories.
- **FR-010**: User-interface code MUST be checked for type errors, lint errors and formatting, and by its test suite; its dependencies MUST be checked against known high-severity advisories.
- **FR-011**: The full Windows app MUST be built (without producing an installer).
- **FR-012**: An architecture check MUST fail when a component depends on one that isn't below it in the crate layout, when a dependency edge isn't listed in the layout, or when a floating-point type appears in the calendar, expression, format or zone components.
- **FR-012a**: The allowed dependency edges MUST be recorded as a machine-readable list inside the knowledge concept `/architecture/crate-layout.md`, and the architecture check MUST read them from there and nowhere else. The design PR for this spec adds that list; the owner re-verifies the concept.
- **FR-013**: Every failing check MUST say what failed and where (file, component or platform).
- **FR-014**: The checks' tests MUST include deliberate failure cases, so a check that silently stopped detecting problems would be noticed.

**Dependency updates**

- **FR-015**: Dependabot MUST propose updates for core and user-interface dependencies weekly, grouped into at most one PR per ecosystem, labelled so the PR policy accepts them.

**Gates**

- **FR-016**: After this feature merges, the owner MUST add the new checks to the required checks on `main`; the spec's tasks MUST include a step telling the owner exactly which check names to add.

### Key Entities

- **Component**: one part of the core, with a name and a position in the layout. It may depend only on components below it.
- **Crate layout**: the agreed list of components and the dependency edges allowed between them. Its single source of truth is the machine-readable list in the knowledge concept `/architecture/crate-layout.md`.
- **Check**: an automated test of the repository, with a name, the platforms it runs on, and a result (passed, failed, skipped).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Starting from a fresh checkout on a Windows machine with the prerequisites installed, the owner can launch the app in under 15 minutes by following the documentation, with no help.
- **SC-002**: After the first build, the app's window appears within 5 seconds of running either command, and in development mode a change to the window's text appears within 3 seconds of saving.
- **SC-003**: 100% of PRs run every applicable check; none are skipped except for code areas that don't exist.
- **SC-004**: Each of the deliberate-failure cases (formatting, lint, failing test, UI type error, broken build, upward dependency, unlisted dependency edge, floating point in an exact component) is caught by its check, with a message that names the problem.
- **SC-005**: The full set of checks on a PR completes within 20 minutes.
- **SC-006**: Dependabot's first scheduled run after merge produces dependency PRs that pass the PR policy without being relabelled.

## Assumptions

- **Windows only for the app**: the desktop app is built and run on Windows only, because the app is a personal tool (ADR 0004) and the owner uses Windows. It must work on Windows 10 (the owner's machine) and Windows 11. Core components are also tested on Linux to keep them portable.
- **No installer or release**: no installer, code signing or auto-update; ADR 0004 rules out binary releases.
- **Placeholder content**: the window's content is a minimal placeholder. Its real layout arrives with later features; spec 003 adds create/open/save.
- **Toolchain**: the stack in `/architecture/stack.md` (Tauri 2, a Rust core, a TypeScript/React UI) is already decided, so this spec doesn't revisit it. The owner's machine doesn't have the Rust toolchain yet; the owner installs it natively by following the documentation (no Docker; see Clarifications).
- **REBOUND is not built here**: the `rebound-sys` component is a placeholder only. Building the C library belongs to Phase 3.
- **Required checks**: the owner adds the new check names to the `main` ruleset after merge, because Claude can't change repository rules (ADR 0007).
- **Dependabot labels**: core and UI dependency updates only change manifests and lock files, so `fast-lane` is correct for them, unlike GitHub Actions updates (PR #8).
