# Decisions

* [ADR 0001: Bind REBOUND](/decisions/0001-bind-rebound.md) - Use the REBOUND C library via FFI for N-body integration.
* [ADR 0002: Hidden absolute timeline](/decisions/0002-hidden-absolute-timeline.md) - GCT is a hidden technical zero; each calendar's epoch is the start of its first era.
* [ADR 0003: Per-body time zones](/decisions/0003-per-body-time-zones.md) - Each body picks its own time zone scheme, and schemes can be mixed.
* [ADR 0004: Public GPL-3.0 repository](/decisions/0004-public-gpl-repo.md) - The source is published in a public GitHub repository under GPL-3.0; no binaries are distributed.
* [ADR 0005: Calendar core first](/decisions/0005-calendar-core-first.md) - Build the calendar engine before any physics.
* [ADR 0008: Physical time and calendar time](/decisions/0008-physical-time-and-calendar-time.md) - Physics works in physical time on GCT; calendars are human conventions mapped onto it and don't have to be astronomically accurate.

# Process

* [ADR 0006: Spec-driven two-PR workflow](/decisions/0006-spec-driven-two-pr-workflow.md) - Every behaviour change goes through a Spec Kit design PR that the owner merges before a build PR can open.
* [ADR 0007: Bot identity and approval gates](/decisions/0007-bot-identity-and-gates.md) - Claude works as a separate bot account; rulesets, CODEOWNERS and the project board enforce the owner's approval steps.
