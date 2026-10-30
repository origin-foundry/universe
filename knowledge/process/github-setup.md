---
type: Playbook
title: GitHub setup
description: One-time steps the owner takes on GitHub to switch on the workflow and its approval gates.
tags: [process, github, setup, phase-0]
status: draft
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T22:40:00Z }
---

# Purpose

Turns on the workflow in [ADR 0006](/decisions/0006-spec-driven-two-pr-workflow.md) and the gates in [ADR 0007](/decisions/0007-bot-identity-and-gates.md). These steps need the owner's own GitHub account, so Claude can't do them. Order matters: `main` must be pushed before the ruleset exists.

# Steps

## 1. Bot account

1. Create a second GitHub account, e.g. `origin-foundry-bot`, with its own email address and two-factor authentication.
2. As the bot, create a **classic** personal access token with scopes `repo`, `workflow` and `project`, expiring in 90 days. (Fine-grained tokens can't reach a repository owned by another personal account.)
3. Keep the token out of the repository. Give it to Claude Code through `.claude/settings.local.json` (gitignored):

   ```json
   { "env": { "GH_TOKEN": "<bot token>" } }
   ```

## 2. Repository

1. As `origin-foundry`, create the **public** repository `origin-foundry/universe` with no README, licence or `.gitignore` (they already exist locally).
2. Push the initial `main` branch from your machine, as yourself.
3. Settings → Collaborators: invite the bot with the **Write** role; accept the invite as the bot.
4. Settings → Code security: turn on secret scanning, push protection and Dependabot alerts.
5. Settings → Actions → General: workflow permissions **Read repository contents**; leave "Allow GitHub Actions to create and approve pull requests" **off**.
6. Settings → General: leave auto-merge **off**.

## 3. Labels

Create the labels the PR policy and templates use:

```bash
gh label create design --color 5319e7 --description "Spec, plan, tasks, knowledge or process change"
gh label create build --color 0e8a16 --description "Implements a merged design PR"
gh label create fast-lane --color fbca04 --description "No behaviour change"
gh label create feature --color 1d76db --description "Feature issue on the project board"
```

## 4. Ruleset on `main`

Settings → Rules → Rulesets → New branch ruleset, targeting the default branch, **Active**:

- Restrict deletions; block force pushes.
- Require a pull request before merging: **1** approval, **require review from Code Owners**, dismiss stale approvals on new pushes, require approval of the most recent push.
- Require status checks to pass: `knowledge` and `policy` now. Add `rust (ubuntu-latest)`, `rust (windows-latest)`, `rust-deps`, `ui` and `app-build (windows)` once the Phase 0 build PR has made them run at least once.
- Bypass list: **Repository admin, for pull requests only**. This lets you merge your own PRs in an emergency, since GitHub won't let you approve them; the bot is never on the bypass list.

## 5. Project board

1. Create a project (table or board) named **Universe** and link it to the repository.
2. **Status** field options, in order: Backlog, Specifying, Spec review, Planning, Design review, Building, PR review, Done.
3. Add fields **Phase** (single select, 0–6), **Area** (single select: time, physics, calendar, language, formats, storage, architecture, product, process) and **Size** (S, M, L).
4. Project settings → Manage access: add the bot with **Write**.
5. Create milestones `Phase 0` to `Phase 6` in the repository.

## 6. First PR

Claude pushes `setup/workflow-guardrails` as the bot and opens it as a **draft** labelled `design`. When `knowledge` and `policy` pass, the ready gate marks it ready and assigns you. Note that `pull_request_target` and `workflow_run` workflows run from `main`, so the policy check and ready gate only take effect once this first PR is merged; review it by hand.

# Done when

- The bot can push branches and open PRs but cannot push to or merge into `main`.
- A draft PR with passing checks is marked ready and assigned to `origin-foundry` automatically.
- A PR without exactly one kind label fails `policy`.
