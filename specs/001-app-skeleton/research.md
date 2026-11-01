# Research: App Skeleton

Decisions for spec 001. Each records what was chosen, why, and what else was considered. Exact tool versions are pinned in the build PR (latest stable at that time) rather than here, so this plan doesn't go stale.

## R1. Repository layout for a Tauri app whose crate isn't in `src-tauri/`

- **Decision**: One npm package at the repository root (`package.json`), holding the UI's dependencies and the Tauri CLI. The UI's sources stay in `ui/` (Vite's `root` is `ui/`). The Tauri app crate is `crates/uni-app`, with `tauri.conf.json` beside its `Cargo.toml`. The Cargo workspace is at the root (`Cargo.toml`), with every crate under `crates/`.
- **Rationale**: Keeps the agreed crate layout (`/architecture/crate-layout.md`) unchanged and gives one place to run commands from. The Tauri CLI finds the app by searching the working directory for `tauri.conf.json`, so running it from the root finds `crates/uni-app`.
- **Risk to verify first in the build**: if the CLI doesn't find `crates/uni-app/tauri.conf.json` from the root, fall back to passing the app directory explicitly; if that isn't supported either, move the app crate to `src-tauri/` and amend the crate layout in a design PR. Task T013 checks this before anything else is built on it.
- **Alternatives**: `src-tauri/` at the root (Tauri's convention, but contradicts the verified crate layout); npm workspaces with a separate `ui/package.json` (two package files for one UI, and Tauri's CLI would still need locating).

## R2. Commands: development and run (FR-004)

- **Decision**: `npm run dev` starts the app in development mode (Vite dev server with hot reload; the core rebuilds when Rust files change). `npm run app` builds the optimised app without an installer and launches it. Both run the prerequisites check first through npm's `predev` / `preapp` hooks.
- **Rationale**: npm scripts work the same in PowerShell, Git Bash and CI. `run` isn't used as the script name because `npm run run` reads badly.
- **Alternatives**: a `Makefile` or `justfile` (another tool to install on Windows); separate PowerShell scripts (not usable from Git Bash or Linux CI).

## R3. Prerequisites and their check (FR-004a)

- **Decision**: `scripts/check_prereqs.py`, using only the standard library. It checks, and reports with required versions and the install step to follow:
  1. Rust toolchain (`rustc`, `cargo`) at the version pinned in `rust-toolchain.toml`, with the MSVC target;
  2. Microsoft C++ Build Tools (found through `vswhere.exe`);
  3. the WebView2 runtime (registry key; it ships with Windows 11 and up-to-date Windows 10);
  4. Node.js 22 or later, and `npm`.
  
  It installs nothing, and exits 1 if anything is missing.
- **Rationale**: Python is already required for the repository's scripts and hooks, so it adds no new prerequisite.
- **TLS interception on the owner's machine**: the setup guide includes a troubleshooting section. Node and npm use their own certificate bundle, so they're told to use the Windows store (`NODE_OPTIONS=--use-system-ca`, available in Node 22). rustup is told to download with the system's own TLS if it reports certificate errors. Cargo on Windows already uses the system store. The check script makes a test HTTPS request through Node and reports a certificate error clearly if one happens.
- **Alternatives**: Docker (rejected in Clarifications); a script that installs everything (rejected by the owner).

## R4. Where the setup guide lives

- **Decision**: `docs/development.md`, linked from the README, written in the build PR.
- **Rationale**: It documents commands that only exist after the build, and the PR policy doesn't allow build PRs to change `knowledge/`. The knowledge bundle keeps describing the system; this guide is operational.
- **Alternatives**: a knowledge playbook (would have to be written before the commands exist, in this design PR).

## R5. Rust toolchain and lints

- **Decision**: `rust-toolchain.toml` pins one exact stable Rust version, with `rustfmt` and `clippy`. Edition 2024. Workspace-wide `[workspace.lints]`: `unsafe_code = "forbid"` (relaxed only in `rebound-sys` from Phase 3), and Clippy warnings denied in CI (`-D warnings`). The four exact crates (`uni-calendar`, `uni-expr`, `uni-format`, `uni-zones`) also turn on Clippy's `float_arithmetic` restriction lint as a second line of defence behind the architecture check.
- **Rationale**: A pinned toolchain means local, CI and Dependabot builds agree. Toolchain bumps are deliberate `fast-lane` PRs.
- **Alternatives**: floating `stable` (CI can break on a new release with no code change).

## R6. Linux CI and the app crate

- **Decision**: On Linux, the Rust job builds and tests the workspace **excluding `uni-app`**. Windows builds and tests everything.
- **Rationale**: `uni-app` depends on Tauri, which on Linux needs WebKitGTK system libraries. The app is Windows-only (spec Assumptions), while the core crates must stay portable.
- **Alternatives**: installing WebKitGTK on the Linux runner (slower, for a platform we don't ship).

## R7. The architecture check (FR-012, FR-012a, FR-013, FR-014)

- **Decision**: `scripts/check_architecture.py`, run in a new CI job named `architecture`.
  1. It reads the allowed-dependency list from the fenced `yaml` block marked `crate-dependencies` in `knowledge/architecture/crate-layout.md`. It fails if the block is missing, malformed, mentions an unknown crate, or contains a cycle.
  2. It gets the real dependencies from `cargo metadata --no-deps` and compares them. It fails on a workspace crate missing from the list, a listed crate that doesn't exist, or a dependency between workspace crates that the list doesn't allow (counting normal, build and dev dependencies).
  3. In the four exact crates, it scans the Rust sources, ignoring comments and string literals, for `f32`/`f64` and float literals (e.g. `1.0`, `2e3`). Each hit is reported as `file:line`.
  
  Messages name the rule broken and the crates or file involved.
- **Tests**: `scripts/tests/test_check_architecture.py` runs the check against small fixture workspaces. There's one fixture that passes, and one for each failure: an upward dependency, an unlisted edge, a missing crate, a cycle, a float type, a float literal, and a float inside a comment or string (which must pass).
- **Rationale**: Python matches the repository's other checks. `cargo metadata` is Cargo's stable, machine-readable view of the workspace.
- **Alternatives**: `cargo-deny`'s ban rules (they can't express "only these edges"); Clippy `disallowed_types` alone (it doesn't cover float literals, and it's configured per crate rather than from the knowledge concept).
- **Guardrail status**: the script enforces constitution principles III and V. It's created in the build PR, so it can't be a guardrail file yet (build PRs may not change guardrails). A follow-up design PR adds it to the policy's guardrail list.

## R8. Allowed dependencies between crates

- **Decision**: these edges, recorded in the knowledge concept (this design PR changes it; see `data-model.md`):

  | Crate | May depend on |
  |---|---|
  | `uni-time` | none |
  | `rebound-sys` | none |
  | `uni-expr` | `uni-time` |
  | `uni-zones` | `uni-time`, `uni-expr` |
  | `uni-calendar` | `uni-time`, `uni-expr`, `uni-zones` |
  | `uni-format` | `uni-time`, `uni-expr`, `uni-zones`, `uni-calendar` |
  | `uni-physics` | `uni-time`, `uni-expr`, `rebound-sys` |
  | `uni-store` | every crate above except `rebound-sys` |
  | `uni-presets` | `uni-time`, `uni-expr`, `uni-zones`, `uni-calendar`, `uni-format`, `uni-physics` |
  | `uni-app` | every crate except `rebound-sys` |

- **Rationale**: this follows the dependency direction in the crate layout. Two choices needed care:
  - **Sky functions.** The expression language needs sky functions (`altitude`, `elongation`…), but physics already depends on expressions for custom events. So `uni-expr` defines a "sky" interface and `uni-physics` implements it, which avoids a cycle.
  - **Zone rules** can mention sky events, through the same interface.
  
  Only `uni-physics` talks to `rebound-sys`.
- **Alternatives**: letting `uni-expr` depend on `uni-physics` (creates a cycle with custom events); a separate `uni-sky` interface crate (one more crate for a single interface).

## R9. The user interface

- **Decision**: React and TypeScript in strict mode, built with Vite. ESLint (flat config with typescript-eslint and the React hooks rules), Prettier, and Vitest with Testing Library. npm scripts: `typecheck`, `lint`, `format:check`, `test`. The window's content is one component, which calls a Tauri command for the app's name and version.
- **Rationale**: These are the tools Tauri's own React template uses. They match the CI job that's already written (`ci.yml`, job `ui`), and Vitest runs in Node without a browser.
- **Alternatives**: Jest (slower and needs more setup with Vite).

## R10. Proving the UI reaches the core (FR-007)

- **Decision**: `uni-app` exposes one Tauri command, `app_info`, which returns `{ name, version }` (the version is the workspace version). The UI shows it, and a UI test mocks the command.
- **Rationale**: It's the smallest real round trip from the UI into Rust and back.

## R11. Window and app identity

- **Decision**: Window title "Universe", 1200×800 by default and resizable. App identifier `io.github.origin-foundry.universe`. Tauri capabilities are limited to `core:default`, plus a strict content security policy. App icons are generated from one placeholder image made in the build (no third-party artwork).
- **Rationale**: Least privilege from the start. The identifier must be reverse-domain style, and the repository's GitHub address is the domain we actually control.

## R12. Dependency checks and Dependabot (FR-009, FR-010, FR-015)

- **Decision**: `deny.toml` at the root, with `cargo-deny` checking advisories, bans (duplicate versions only warn) and sources (crates.io only). It doesn't check licences, because the whole project is GPL-3.0. On the UI side, `npm audit --audit-level=high` (already in `ci.yml`). Dependabot gets `cargo` and `npm` entries, weekly, each with one group matching everything, labelled `fast-lane`.
- **Rationale**: Dependency-only PRs touch manifests and lock files, which the policy allows in `fast-lane`.

## R13. Making the new checks required (FR-016)

- **Decision**: After the build PR merges, the owner adds these required checks to the `main` ruleset: `rust (ubuntu-latest)`, `rust (windows-latest)`, `rust-deps`, `ui`, `app-build (windows)` and `architecture`. The final task in `tasks.md` hands this to the owner with step-by-step instructions.

## R14. Stop hook

- **Decision**: In this design PR, the Stop hook also runs `scripts/check_architecture.py` when both it and `Cargo.toml` exist. The hook is a guardrail file, so it can only change in a design PR. It stays a no-op until the build PR adds the script.
