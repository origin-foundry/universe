# Bundle update log

## 2026-10-30
* **Verification**: human:origin-foundry reviewed the whole bundle area by area; all 41 concepts are now `status: stable` and verified. Changes made during review: moved [light-delay queries](/time/light-delay.md) from formats to time; test dates in [presets and validation](/product/presets-and-validation.md) are generated from an independent open-source implementation; workflow setup folded into Phase 0 of the [delivery phases](/product/delivery-phases.md).
* **Update**: Rewrote [ADR 0004](/decisions/0004-public-gpl-repo.md) for a public GPL-3.0 repository (previously personal use, not distributed), and updated [ADR 0001](/decisions/0001-bind-rebound.md) and the [overview](/overview.md) to match.
* **Creation**: Added process decisions [ADR 0006](/decisions/0006-spec-driven-two-pr-workflow.md) (spec-driven two-PR workflow) and [ADR 0007](/decisions/0007-bot-identity-and-gates.md) (bot identity and approval gates).
* **Initialization**: Created the bundle by splitting DESIGN.md (Draft 2) into concepts. All concepts start as `status: draft` and unverified, pending human review.
