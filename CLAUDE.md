# Universe

A desktop app for defining physically grounded, non-Earth calendars across multiple star systems. Tauri 2, Rust core, TypeScript/React UI. GPL-3.0, public repo `origin-foundry/universe`.

## Read first

- **Rules:** `.specify/memory/constitution.md`. Every plan must comply; violations need a written justification.
- **Design:** `knowledge/index.md` (an Open Knowledge Format v0.2 bundle). This is the source of truth for what the system is. `DESIGN.md` is superseded; never edit it.
- **Workflow:** `knowledge/decisions/0006-spec-driven-two-pr-workflow.md` and `knowledge/decisions/0007-bot-identity-and-gates.md`.

## How work flows

1. A feature starts as a GitHub issue on the project board.
2. Spec Kit skills (in `.claude/skills/`): `/speckit-specify` → `/speckit-clarify` → (owner reviews the spec) → `/speckit-plan` → `/speckit-tasks` → `/speckit-analyze`.
3. **Design PR** (label `design`): `specs/NNN-name/` plus any `knowledge/` updates. The owner merges it.
4. `/speckit-implement` on a new branch → **build PR** (label `build`), body contains `Design: #<design PR number>`.
5. Changes with no behaviour change may go straight to a PR labelled `fast-lane`.

Tasks stay in `tasks.md`; they are not turned into GitHub issues (one issue per feature). Don't use `/speckit-constitution` to rewrite the constitution: amendments go through a design PR the owner approves.

Spec Kit is installed with uv. This machine intercepts TLS, so uv needs `--system-certs`, e.g. `uv tool upgrade --system-certs specify-cli`.

Ask the owner questions one at a time with options to choose from, never as a list.

## Hard rules

- Never push to `main`, merge a PR, or touch the repository ruleset.
- Open PRs as drafts. CI marks them ready and assigns the owner when all checks pass.
- Only move board cards into review columns, never out of them.
- Never add `verified:` to a knowledge concept or change the constitution without the owner's explicit approval.
- Never commit secrets.

## Checks

- Knowledge bundle: `python scripts/check_okf.py knowledge`
- PR policy (runs in CI): `scripts/check_pr_policy.py`
- A Stop hook (`scripts/hooks/stop-checks.sh`) runs the fast local checks before a turn ends; fix what it reports rather than working around it.

## Conventions

- Bundle concepts: one file per concept, frontmatter with `type`, `title`, `description`, `status`, `generated`; absolute links (`/area/concept.md`); list every concept in its folder's `index.md` with its description; add a `log.md` entry for every change.
- Commit messages: imperative subject line, body explaining why.
