---
type: Playbook
title: GitHub setup
description: One-time steps the owner takes on GitHub to switch on the workflow and its approval gates.
tags: [process, github, setup, phase-0]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-31T12:00:00Z }
verified: { by: human:origin-foundry, at: 2026-11-01T08:27:18Z }
---

# Purpose

Turns on the workflow in [ADR 0006](/decisions/0006-spec-driven-two-pr-workflow.md) and the gates in [ADR 0007](/decisions/0007-bot-identity-and-gates.md). First carried out on 2026-10-30/31; kept for token rotation and for rebuilding the setup. Order matters: `main` must be pushed before the ruleset exists, and required status checks are added only after the setup PR is merged.

# Steps

## 1. Bot account (owner, private browser window)

1. Sign up at github.com/signup as `origin-foundry-bot`, with its own email (e.g. a `+bot` plus-address). Turn on two-factor authentication with an authenticator app; save the recovery codes.
2. After step 3 below, create a **classic** token as the bot: Settings → Developer settings → Personal access tokens → Tokens (classic). Scopes `repo`, `workflow`, `project` only; 90-day expiry. Fine-grained tokens can't reach a repository owned by another personal account.

## 2. Token on this machine (owner)

The token lives in `.claude/settings.local.json` (gitignored), which Claude Code loads as environment variables, so `gh` and git inside Claude Code act as the bot.

1. `copy .claude\settings.local.example.json .claude\settings.local.json`
2. `notepad .claude\settings.local.json` and replace `PASTE_BOT_TOKEN_HERE`. Never edit the template itself.
3. **Never paste the token into a Claude conversation**, and don't edit a file Claude has written or read in the current session: the app shows Claude the new contents of such files. Copying the template avoids this. If a token is ever shown, regenerate it.
4. Claude verifies it without displaying it: the account should be `origin-foundry-bot` with push but not admin access.

Settings are picked up mid-session, so from then on Claude acts as the bot. Claude never drops the bot token to act as the owner (see CLAUDE.md): owner-only steps are done by the owner. During the original setup the owner explicitly asked Claude to script the board with their login, before this rule existed; in a terminal of your own, `gh` uses your login anyway.

This repo's git is configured to take credentials from `gh` rather than Windows Credential Manager, so pushes from Claude Code go out as the bot and pushes from your own terminals go out as you:

```bash
git config --local credential.https://github.com/.helper ""
```

```bash
git config --local --add credential.https://github.com/.helper "!gh auth git-credential"
```

## 3. Repository (owner)

1. Create the **public** repository `origin-foundry/universe` with nothing initialised (no README, licence or `.gitignore`).
2. Push `main` from your own terminal: `git push -u origin main`.
3. Settings → Collaborators: add `origin-foundry-bot` (write access on a personal repo); accept the invite as the bot.
4. Settings → Advanced Security: Dependabot alerts, Dependabot security updates, secret scanning and push protection on.
5. Settings → Actions → General: workflow permissions **read**; "Allow GitHub Actions to create and approve pull requests" **off**; fork PR workflows from outside collaborators **require approval**.
6. Settings → General → Pull Requests: auto-merge **off**; automatically delete head branches **on**.

## 4. Labels (bot)

`design`, `build`, `fast-lane`, `feature`, created with `gh label create` by the bot.

## 5. Ruleset on `main` (owner)

Settings → Rules → Rulesets → New branch ruleset `main`, **Active**, targeting the default branch:

- Bypass: **Repository admin, for pull requests only** (GitHub won't let you approve your own PRs; the bot is never on the bypass list).
- Restrict deletions; block force pushes.
- Require a pull request: 1 approval, dismiss stale approvals, require Code Owner review, require approval of the most recent push, require conversation resolution.
- **No required status checks until the setup PR is merged.** `policy` runs from `main` via `pull_request_target`, so it can't report on the PR that introduces it. After the merge, add `knowledge` and `policy`; add `rust (ubuntu-latest)`, `rust (windows-latest)`, `rust-deps`, `ui` and `app-build (windows)` once the Phase 0 build PR has run them.

Claude can check the result through the read-only `rules/branches/main` API; the guard blocks the `rulesets` API entirely.

## 6. Project board (owner)

1. From your own terminal, `gh project create --owner "@me" --title Universe`. (`gh project link` fails for personal accounts, so the repo is linked with the `linkProjectV2ToRepository` GraphQL mutation.)
2. Fields: Phase (0–6), Area (bundle areas plus `process`), Size (S/M/L).
3. Status options replaced through `updateProjectV2Field`: Backlog, Specifying, **Spec review**, Planning, **Design review**, Building, **PR review**, Done.
4. Owner, in the UI: project ⋯ → Settings → Manage access → add `origin-foundry-bot` as **Write**. Granting access is the owner's own decision, never Claude's.
5. Milestones `Phase 0` to `Phase 6`, created by the bot.

## 7. First PR (bot)

Claude pushes `setup/workflow-guardrails` as the bot and opens it as a **draft** labelled `design`, with a feature issue on the board in Design review. The policy check and ready gate only run once this PR is on `main`, so the owner reviews it fully by hand, marks it ready, approves and merges it, then adds the required status checks.

# Done when

- The bot can push branches and open PRs but can't push to or merge into `main`.
- After the setup PR is merged, a draft PR with passing checks is marked ready and assigned to `origin-foundry` automatically.
- A PR without exactly one kind label fails `policy`.
