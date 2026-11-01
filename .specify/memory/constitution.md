# Universe Constitution

The non-negotiable rules for every change to Universe. Spec Kit's `/speckit.plan` step checks each plan against these principles, and any violation must be justified in the plan's Complexity Tracking table or the plan is rejected.

Design detail lives in the knowledge bundle (`knowledge/`); this document holds only the rules.

## Core Principles

### I. Spec before code

Every behaviour change starts as a Spec Kit spec and goes through two PRs: a **design PR** (spec, plan, tasks, knowledge updates) that the owner merges, then a **build PR** that links to it. Only changes with no behaviour change (typos, docs, formatting, CI tweaks, dependency bumps, pure refactors) may use the **fast lane**, and they still need passing checks and the owner's approval. See `knowledge/decisions/0006-spec-driven-two-pr-workflow.md`.

### II. The knowledge bundle is the source of truth

`knowledge/` describes what the system is. Every spec cites the concepts it relies on or alters. A change that alters the design updates the affected concepts and adds a `knowledge/log.md` entry in its design PR. Concepts are `status: draft` until the owner approves them; only the owner's explicit approval in conversation or on a PR may add a `verified` entry with `human:origin-foundry`. The bundle must always pass `scripts/check_okf.py`.

### III. Exact calendar arithmetic

Calendar, expression, format and zone code (`uni-calendar`, `uni-expr`, `uni-format`, `uni-zones`) uses integers and exact rationals only, never floating point. Absolute instants are `Instant { secs: i64, nanos: u32 }`. Floating point is confined to physics, and crosses into exact types at one tested boundary.

### IV. The absolute timeline stays hidden

GCT exists only to equate points on different calendars. The UI never shows raw GCT or Earth units unless the user asks; the only exception is a developer/debug panel. See `knowledge/decisions/0002-hidden-absolute-timeline.md`.

### V. Layering

Crate dependencies only point downward, as listed in `knowledge/architecture/crate-layout.md`. Physics never depends on calendars. Core crates never depend on Tauri. A new crate or a new dependency edge between crates is a design change.

### VI. Test-first correctness

- Engine code is written test-first: tests that fail, then the code that passes them.
- Calendar presets are checked against known dates generated from an independent open-source implementation, with the generator, library and version committed. Published reference tables are never copied.
- Every calendar and every format has round-trip property tests (date → instant → date; render → parse).
- Every bug fix adds a regression test that fails before the fix.

### VII. Never resolve ambiguity silently

Invalid dates, skipped or repeated local times, ambiguous parses and unreliable far-future physics are reported to the user with the reason. Nothing is silently wrapped, rounded or guessed.

### VIII. Small, reviewable changes

One spec produces one build PR, reviewable in about 30 minutes. If a plan's task list would exceed that, the spec is split before the design PR is opened.

## Human Gates

These rules bind Claude (and any other agent) working on this repository:

- Never push to `main`, never merge a PR, never bypass or edit the repository ruleset.
- Work as the bot account, never with the owner's credentials.
- Open PRs ready for review, not as drafts. The ready gate assigns the owner once every check passes; until then the PR isn't the owner's to review.
- On the project board, move cards **into** review columns only. Only the owner moves cards **out** of Spec review, Design review and PR review.
- Never mark knowledge concepts verified, or change this constitution, without the owner's explicit approval.
- Never commit secrets. The repository is public.

## Quality Gates

A PR is ready for the owner only when every required check passes: Rust (format, lint with warnings as errors, tests on Windows and Linux), UI (types, lint, format, tests), app build on Windows, knowledge bundle validation, PR policy (classification, spec traceability, task completion), architecture rules, and dependency advisories. Checks that don't apply yet (for example, before any Rust code exists) are skipped, not removed.

## Licensing

The repository is GPL-3.0. Vendored third-party code keeps its own licence notice. See `knowledge/decisions/0004-public-gpl-repo.md`.

## Governance

This constitution overrides any other practice in the repository. Amendments go through a design PR that the owner merges, update the version below (major: a principle removed or redefined; minor: a principle added or materially expanded; patch: clarifications), and update any affected templates or knowledge concepts in the same PR.

**Version**: 1.0.1 | **Ratified**: 2026-11-01 | **Last Amended**: 2026-11-01

- 1.0.1: PRs open ready for review instead of as drafts; the ready gate assigns the owner when every check passes.
- 1.0.0: Ratified by merging #2.
