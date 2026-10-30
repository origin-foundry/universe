---
type: Risk Register
title: Risks
description: Known project risks and how the design mitigates them.
tags: [risks]
status: stable
generated: { by: claude-code/claude-opus-5-5, at: 2026-10-30T08:24:17Z }
verified: { by: human:origin-foundry, at: 2026-10-30T22:09:07Z }
---

# Risks

1. **Scope.** N-body physics plus relativistic trajectories across multiple systems is a lot of work. The [delivery phases](/product/delivery-phases.md) keep each stage usable on its own.
2. **Hebrew-calendar-level complexity** drives [expression language](/language/expression-language.md) features. Keeping the language small keeps it safe and fast, but exotic calendars may need custom functions.
3. **Parsing arbitrary formats** can be ambiguous or impossible to invert. Mitigated by the invertibility check and render-only fallback in [format parsing](/formats/parsing.md).
4. **Chaotic dynamics** make far-future dates in some systems meaningless. Mitigated by the reliability indicator in [integration](/physics/integration.md).
5. **SQLite as the primary format** isn't diff-friendly. Mitigated by [JSON export](/storage/export.md).
6. **REBOUND build on Windows.** It's developed mainly on Linux/macOS; compiling with MSVC via `cc` may need patches. Checked early in phase 3, with MinGW as a fallback. See [ADR 0001](/decisions/0001-bind-rebound.md).
