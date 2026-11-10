# Bundle update log

## 2026-11-10
* **Creation**: Added [ADR 0008: Physical time and calendar time](/decisions/0008-physical-time-and-calendar-time.md) (draft), the ground rules agreed in the review of spec 004: physics works in physical time, calendars are conventions mapped onto it and don't have to be astronomically accurate.
* **Update**: Spec 004 (calendar engine basics): [unit kinds](/calendar/unit-kinds.md) gains member units, the per-unit first number and the single-chain first engine; [conversion](/calendar/conversion.md) gains the next-whole-nanosecond rule, reported with the exact amount; [eras and epoch](/calendar/eras-and-epoch.md) says an era numbers a chosen unit, with units above it numbered by count from the epoch. The [glossary](/glossary.md) renames **Cycle** to **Physical cycle** and adds physical time, calendar time, member unit, first number, era's unit and astronomical year numbering. All back to draft until re-verified by human:origin-foundry.

## 2026-11-04
* **Update**: The [feature workflow](/process/feature-workflow.md) gains the merge-method rule and "Keeping the record durable" (protecting against lost conversation context). Re-verified by human:origin-foundry in review of #28.
* **Update**: Spec 003 (project file): [storage schema](/storage/schema.md) gains the application identifier value, versioning and migrations, the in-memory working copy, crash-safe saves and file locking, and notes what schema version 1 contains. Re-verified by human:origin-foundry in review of #25.

## 2026-11-02
* **Update**: Spec 002 (exact time core): `uni-time` joins the exact crates in the [crate layout](/architecture/crate-layout.md), and [timeline and frames](/time/timeline-and-frames.md) says the floating-point boundary for physics lives in `uni-physics`. Both re-verified by human:origin-foundry in review of #17.

## 2026-11-01
* **Creation**: Added the [feature workflow](/process/feature-workflow.md) playbook: the path from Backlog to Done, roles, and who moves each board card. Verified by human:origin-foundry in review of #13.
* **Update**: Spec 001 (app skeleton): [crate layout](/architecture/crate-layout.md) gains the machine-readable list of allowed dependencies that the architecture check enforces, the `exact` crates, the sky-function interface rule, and the root `package.json`. Re-verified by human:origin-foundry in review of #12 (recorded in #13, because the commit arrived after #12 merged).
* **Update**: PRs are opened ready for review instead of as drafts, because the Actions token can't take a PR out of draft; the ready gate now assigns the owner and comments when every check passes. Amended [ADR 0007](/decisions/0007-bot-identity-and-gates.md) (also correcting its PR-policy description) and the [GitHub setup](/process/github-setup.md) playbook; both re-verified by human:origin-foundry in review of #7.
* **Verification**: human:origin-foundry reviewed PR #2 and verified the [GitHub setup](/process/github-setup.md) playbook.

## 2026-10-30
* **Creation**: Added the [GitHub setup](/process/github-setup.md) playbook (draft) for switching on the workflow and its gates.
* **Verification**: human:origin-foundry reviewed the whole bundle area by area; all 41 concepts are now `status: stable` and verified. Changes made during review: moved [light-delay queries](/time/light-delay.md) from formats to time; test dates in [presets and validation](/product/presets-and-validation.md) are generated from an independent open-source implementation; workflow setup folded into Phase 0 of the [delivery phases](/product/delivery-phases.md).
* **Update**: Rewrote [ADR 0004](/decisions/0004-public-gpl-repo.md) for a public GPL-3.0 repository (previously personal use, not distributed), and updated [ADR 0001](/decisions/0001-bind-rebound.md) and the [overview](/overview.md) to match.
* **Creation**: Added process decisions [ADR 0006](/decisions/0006-spec-driven-two-pr-workflow.md) (spec-driven two-PR workflow) and [ADR 0007](/decisions/0007-bot-identity-and-gates.md) (bot identity and approval gates).
* **Initialization**: Created the bundle by splitting DESIGN.md (Draft 2) into concepts. All concepts start as `status: draft` and unverified, pending human review.
