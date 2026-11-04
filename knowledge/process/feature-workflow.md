---
type: Playbook
title: Feature workflow
description: The normal path of a feature from Backlog to Done, who does each step, and who moves the board card.
tags: [process, workflow, board, roles]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-11-01T15:00:00Z }
verified: { by: human:origin-foundry, at: 2026-11-04T13:22:06Z }
---

# Roles

| Role | Who | Does |
|---|---|---|
| **Owner** | `origin-foundry` | Chooses work, approves specs and designs, reviews and merges every PR, does owner tasks, moves cards **out of** review columns. |
| **Claude** | the bot account `origin-foundry-bot` | Writes specs, plans, tasks and code; opens PRs; moves cards **into** review columns. Never merges, never moves a card out of a review column. |
| **Automation** | GitHub Actions and the board's built-in workflows | Runs checks, hands green PRs to the owner (the ready gate), moves cards to Done when an issue closes or a PR merges. |

These rules come from [ADR 0006](/decisions/0006-spec-driven-two-pr-workflow.md) and [ADR 0007](/decisions/0007-bot-identity-and-gates.md).

# The path

One issue per feature. Its card moves through the board's Status columns:

| Column | Whose turn | What happens | Card moves on when | Moved by |
|---|---|---|---|---|
| **Backlog** | Owner | The owner picks the next feature to work on. | The owner asks Claude to start it | Claude, to Specifying |
| **Specifying** | Claude | Claude writes `spec.md` with `/speckit-specify`, then asks clarifying questions one at a time (`/speckit-clarify`). | The spec is ready to review | Claude, to Spec review |
| **Spec review** | Owner | The owner reads the spec and approves it, or asks for changes, in conversation. | The owner approves | **Owner**, to Planning |
| **Planning** | Claude | Claude writes the plan, research, contracts and tasks (`/speckit-plan`, `/speckit-tasks`), cross-checks them (`/speckit-analyze`), and opens the **design PR**. | The design PR is open | Claude, to Design review |
| **Design review** | Owner | The ready gate assigns the owner when the checks pass. The owner reviews; any knowledge concept that changed is re-verified in conversation first, and the owner waits for Claude to confirm the verification commit is pushed; then the owner approves and merges the design PR. | The design PR is merged | **Owner**, to Building |
| **Building** | Claude, plus owner tasks | Claude works through `tasks.md` on a build branch, ticking tasks as they're done. Tasks marked **(owner)** are handed to the owner and ticked once the owner confirms. Claude opens the **build PR**. | The build PR is open | Claude, to PR review |
| **PR review** | Owner | The ready gate assigns the owner when the checks pass. The owner reviews, approves and merges the build PR. | The build PR is merged | Automatic |
| **Done** | Automation | Merging the build PR closes the issue (`Closes #N`); the board's "Pull request merged" and "Item closed" workflows move the card here. | | |

## Merging

- **Design and build PRs**: **Create a merge commit**. History keeps each task, review fix and verification commit, so it shows how a feature was built and when each approval landed.
- **Fast-lane PRs**: **Squash and merge**. Each small change becomes one commit on `main` that names its PR.

GitHub can't enforce a method per label, so this is a convention, recorded here and in `CLAUDE.md`.

## Keeping the record durable

Long conversations with Claude get condensed, and anything that exists only in the conversation can be lost. Claude therefore gives every pitfall, workaround, decision or fragile manual step a durable home as part of the work, without being asked:

- the working notes in `CLAUDE.md`;
- a script that does the fiddly part (for example, `scripts/board.py` for board moves);
- a knowledge concept or ADR;
- or an automated check that enforces it.

Claude says when it has done so. Each spec also starts in a fresh Claude session, which reads `CLAUDE.md`, this bundle and the spec's `tasks.md` from scratch.

## How the owner knows it's their turn

- **In conversation**: Claude asks a question with options (spec approval, owner tasks, design approval).
- **On GitHub**: the ready gate **assigns the owner** and comments "All checks passed". GitHub also requests the owner's review as soon as a PR opens; that request alone doesn't mean the checks have passed.
- **On the board**: a card in Spec review, Design review or PR review is waiting for the owner.

## Shortcuts

- **Fast lane**: changes with no behaviour change (typos, docs, formatting, CI tweaks, dependency bumps) skip the spec and design PR. They go straight to a PR labelled `fast-lane`, which the owner reviews and merges.
- **A stage already done**: if a card is still in an earlier review column when its stage is finished (for example, Spec review after the design PR has merged), the owner moves it straight to where the work now is.

## When things change mid-way

- **A finding in the build changes the design**: the build stops, and Claude opens a new design PR. Build PRs can't change specs or knowledge.
- **The owner wants changes in review**: Claude makes them on the same branch. If a knowledge concept changes, it goes back to draft until the owner re-verifies it.
- **After merge**: steps that can only happen after merging (for example, adding required checks) are listed in `tasks.md` under "After merge", outside the checklist.
