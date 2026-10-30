---
type: Architecture Decision
title: "ADR 0004: Public GPL-3.0 repository"
description: The source is published in a public GitHub repository under GPL-3.0; no binaries are distributed.
tags: [decision, licensing, repository]
status: stable
decided: 2026-10-30
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T09:10:00Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:14:45Z }
---

# Context

The app was originally intended for personal use only and not distributed. On a personal GitHub Free account, enforcing approval steps on `main` (rulesets, required checks, required reviews) is only available for public repositories. That enforcement is central to the [spec-driven workflow](/decisions/0006-spec-driven-two-pr-workflow.md).

# Decision

- The repository (`origin-foundry/universe`) is **public**.
- The whole repository is licensed **GPL-3.0**, matching REBOUND's licence, so vendoring REBOUND's source is clean. See [ADR 0001](/decisions/0001-bind-rebound.md).
- The app remains a personal tool: no binary releases are published. Nothing in this decision commits to supporting outside users.

# Consequences

- Anyone may read and reuse the code, under GPL-3.0 terms only.
- Required licence notices (a `LICENSE` file, and REBOUND's own notice kept in `rebound-sys/`) must be present.
- Nothing secret may be committed. Secret scanning and push protection are enabled.
- If binaries are ever shared, they are GPL-3.0 and must come with source; this is already satisfied by the public repo.
