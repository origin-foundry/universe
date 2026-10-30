---
type: Process Decision
title: "ADR 0006: Spec-driven two-PR workflow"
description: Every behaviour change goes through a Spec Kit design PR that the owner merges before a build PR can open.
tags: [decision, process, spec-kit, workflow]
status: stable
decided: 2026-10-30
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T09:10:00Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:14:45Z }
---

# Context

The owner wants to move away from commissioning work from Claude in one shot, while keeping control at defined points.

# Decision

## Spec Kit

[GitHub Spec Kit](https://github.com/github/spec-kit) (installed only from the official repository) structures each feature: constitution → specify → clarify → plan → tasks → analyze → implement. Each feature lives in `specs/NNN-name/`.

## Two PRs per feature

1. **Design PR:** `spec.md`, `plan.md`, `tasks.md`, and any changes to this knowledge bundle. Merging it is the owner's sign-off on the design.
2. **Build PR:** the code. It must link to the merged design PR, every task in `tasks.md` must be ticked, and CI checks both.

## Relationship to this bundle

- The bundle describes what the system *is*; specs describe what a change *does*.
- Every spec cites the concepts it relies on or alters.
- When a change alters the design, the design PR updates the affected concepts and adds a `log.md` entry.

## Fast lane

Changes with **no behaviour change** may skip the design PR: typos, docs, comments, formatting, CI tweaks, dependency bumps, and pure refactors. They still need passing checks and the owner's approval, and carry a `fast-lane` label.

## Where Claude works

In **local Claude Code sessions** on the owner's machine. No Claude GitHub Action.

## Review

**No automated AI review** on PRs for now. Claude does a review pass locally before opening a PR. Revisit if quality slips.

# Consequences

- More PRs per feature, but every design and every merge has an explicit human approval.
- Specs must be small: one spec ≈ one build PR, reviewable in about 30 minutes. Each [delivery phase](/product/delivery-phases.md) is several specs.
- Enforcement mechanics are in [ADR 0007](/decisions/0007-bot-identity-and-gates.md).
