---
type: Process Decision
title: "ADR 0007: Bot identity and approval gates"
description: Claude works as a separate bot account; rulesets, CODEOWNERS and the project board enforce the owner's approval steps.
tags: [decision, process, github, security]
status: draft
decided: 2026-10-30
amended: 2026-11-01
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T09:10:00Z }
---

# Context

GitHub doesn't let an author approve their own PR. If Claude opened PRs with the owner's credentials, a "requires owner approval" rule would block every merge.

# Decision

## Identity

- Claude pushes branches and opens PRs as a **separate bot account** (created by the owner, e.g. `origin-foundry-bot`), added as a collaborator with write access.
- The owner is `origin-foundry`. Human sign-offs in this bundle use the actor `human:origin-foundry`.

## Enforcement on `main`

A repository ruleset (available because the repo is public; see [ADR 0004](/decisions/0004-public-gpl-repo.md)):

- no direct pushes; changes only via PR
- required status checks must pass (see below)
- at least one approval, from a code owner
- the bot can't bypass the ruleset or merge

`CODEOWNERS` names `@origin-foundry` as owner of everything, so every PR needs the owner's approval. `specs/`, `knowledge/` and the constitution are called out explicitly.

## Required checks

Rust (format, lint, tests on Windows and Linux), UI (types, lint, format, tests), app build on Windows, knowledge bundle validation (`scripts/check_okf.py`), PR policy (exactly one of `design`, `build`, `fast-lane`; build PRs link a merged design PR, have all its tasks ticked and don't change design or guardrail files; guardrail files change only in design PRs), architecture rules (dependency direction, no `f64` in calendar crates), dependency advisories, plus GitHub secret scanning and Dependabot.

## Handing PRs to the owner

PRs are opened **ready for review**, not as drafts: the Actions token isn't allowed to take a PR out of draft (found on the first gated PR, 2026-11-01). GitHub requests the code owner's review as soon as a PR opens. The **ready gate** workflow then waits until every check on the PR's latest commit has passed, and only then **assigns the owner** and posts an "All checks passed" comment. The assignment is the signal that it's the owner's turn; a PR that isn't assigned yet isn't ready.

## Project board

One GitHub Project, one issue per feature, phases as milestones. Columns: Backlog → Specifying → **Spec review** → Planning → **Design review** → Building → **PR review** → Done.

Claude (as the bot) creates issues and moves cards **into** the review columns (bold). Only the owner moves cards **out** of them.

# Consequences

- The owner must create the bot account and give it a token with repo and project scopes.
- Every merge to `main` has the owner's approval, enforced by GitHub rather than by convention.
- The workflow is described in [ADR 0006](/decisions/0006-spec-driven-two-pr-workflow.md).
