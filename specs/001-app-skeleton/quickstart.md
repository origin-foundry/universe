# Quickstart: validating the App Skeleton

How to prove the build PR meets the spec. Commands run from the repository root.

## 1. Fresh checkout to running app (US1, SC-001, SC-002)

1. On a Windows 10 or 11 machine, follow `docs/development.md` to install the prerequisites. Time it; the target is under 15 minutes in total (SC-001).
2. `python scripts/check_prereqs.py` should report every item as `ok`.
3. `npm ci`
4. `npm run dev` should open a window titled **Universe** showing the name, the version and "No universe open".
5. Change the "No universe open" text in the UI source and save. The window should update within 3 seconds without restarting (SC-002).
6. Close the window. The terminal should show no errors.
7. `npm run app` should build the optimised app and open the same window.
8. Uninstall or rename one prerequisite (for example, temporarily take Node off `PATH`) and run `npm run dev`. It should stop with the prerequisites report naming what's missing.

## 2. Checks run on a PR (US2, SC-003, SC-005)

1. Open the build PR. Every job should run: `rust (ubuntu-latest)`, `rust (windows-latest)`, `rust-deps`, `ui`, `app-build (windows)`, `architecture`, `knowledge` and `policy`. None should be skipped, and all should finish within 20 minutes.
2. In a throwaway branch, introduce each of these deliberate failures one at a time and push. Each should fail its job with a message naming the problem (SC-004):

   | Deliberate failure | Job that fails |
   |---|---|
   | Badly formatted Rust | `rust (…)` |
   | An unused variable (a Clippy warning) | `rust (…)` |
   | A failing Rust test | `rust (…)` |
   | A TypeScript type error | `ui` |
   | A failing UI test | `ui` |
   | A syntax error in `crates/uni-app` | `app-build (windows)` and `rust (windows-latest)` |
   | `uni-physics` depending on `uni-calendar` | `architecture` |
   | `uni-time` depending on `uni-expr` | `architecture` |
   | `let x: f64 = 1.0;` in `uni-calendar` | `architecture` (and Clippy's float lint) |

3. The same failures are also covered automatically by `python -m unittest discover -s scripts/tests`, using fixture workspaces, so the check itself is tested (FR-014).

## 3. Architecture rules from the knowledge bundle (US3)

- `python scripts/check_architecture.py` passes on the unchanged repository.
- Editing the `crate-dependencies` block in `knowledge/architecture/crate-layout.md` so it no longer matches the workspace makes the check fail. (Such an edit would itself need a design PR.)

## 4. Dependabot (US4, SC-006)

- After merge, `.github/dependabot.yml` lists `cargo`, `npm` and `github-actions`.
- At Dependabot's next weekly run there should be at most one `cargo` PR and one `npm` PR, each labelled `fast-lane`, and each should pass `policy` without being relabelled.

## 5. Required checks (FR-016)

- After merge, the owner adds the six new check names to the `main` ruleset, following the final task in `tasks.md`.
