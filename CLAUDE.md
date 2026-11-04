# Universe

A desktop app for defining physically grounded, non-Earth calendars across multiple star systems. Tauri 2, Rust core, TypeScript/React UI. GPL-3.0, public repo `origin-foundry/universe`.

## Read first

- **Rules:** `.specify/memory/constitution.md`. Every plan must comply; violations need a written justification.
- **Design:** `knowledge/index.md` (an Open Knowledge Format v0.2 bundle). This is the source of truth for what the system is. `DESIGN.md` is superseded; never edit it.
- **Workflow:** `knowledge/process/feature-workflow.md` (the step-by-step path, roles and who moves each board card), from `knowledge/decisions/0006-spec-driven-two-pr-workflow.md` and `knowledge/decisions/0007-bot-identity-and-gates.md`.

## How work flows

1. A feature starts as a GitHub issue on the project board.
2. Spec Kit skills (in `.claude/skills/`): `/speckit-specify` → `/speckit-clarify` → (owner reviews the spec) → `/speckit-plan` → `/speckit-tasks` → `/speckit-analyze`.
3. **Design PR** (label `design`): `specs/NNN-name/` plus any `knowledge/` updates. The owner merges it.
4. `/speckit-implement` on a new branch → **build PR** (label `build`), body contains `Design: #<design PR number>`.
5. Changes with no behaviour change may go straight to a PR labelled `fast-lane`.

Tasks stay in `tasks.md`; they are not turned into GitHub issues (one issue per feature). Don't use `/speckit-constitution` to rewrite the constitution: amendments go through a design PR the owner approves.

Ask the owner questions one at a time with options to choose from, never as a list.

**Merging** (the owner merges, Claude never does): design and build PRs use **Create a merge commit**, which keeps each task, review fix and verification commit in history. Fast-lane PRs use **Squash and merge**: one commit per small change. When a design PR changes a knowledge concept, the owner re-verifies it in conversation; Claude pushes the verification commit and says "pushed" before the owner merges.

## Hard rules

- Never push to `main`, merge a PR, or touch the repository ruleset.
- Open PRs ready for review (never drafts). The ready gate assigns the owner when every check passes; don't assign or ping the owner yourself.
- Only move board cards into review columns, never out of them.
- Never add `verified:` to a knowledge concept or change the constitution without the owner's explicit approval.
- Never commit secrets.
- Never put personal or machine-specific information in anything public: repo files, commit messages, PR descriptions, issues or comments. That means names, email addresses, paths containing a user folder, hostnames, hardware, network or security setup, and account details. Put machine-only notes in `CLAUDE.local.md`, and describe such things generically in public text. `scripts/check_privacy.py` enforces the patterns it can recognise.
- Never read, print or ask for the bot token (`.claude/settings.local.json`). Never act as the owner, e.g. by dropping `GH_TOKEN` to use their stored login: owner-only actions (repo settings, rulesets, access grants, merges) are done by the owner.

## Checks

- Knowledge bundle: `python scripts/check_okf.py knowledge`
- PR policy (runs in CI): `scripts/check_pr_policy.py`
- Architecture rules: `python scripts/check_architecture.py`
- Privacy: `python scripts/check_privacy.py`. It also reads `.privacy-words` (gitignored, owner-only: one private word or phrase per line, such as a name) when present.
- A Stop hook (`scripts/hooks/stop_checks.py`) runs the fast local checks before a turn ends; fix what it reports rather than working around it.

## Board

Use `scripts/board.py`, never hand-written GraphQL (`gh project` doesn't work for a personal account's project):

- `python scripts/board.py show 11`
- `python scripts/board.py move 11 "Design review"`
- `python scripts/board.py add 24 --status Backlog --phase "0. Foundations" --area storage --size S`

It refuses to move a card out of a review column, because those moves are the owner's. Test changes to it with `scripts/tests/test_board.py` only, never by moving real cards: a "test" move once moved #11 out of Building by mistake.

## Conventions

- Bundle concepts: one file per concept, frontmatter with `type`, `title`, `description`, `status`, `generated`; absolute links (`/area/concept.md`); list every concept in its folder's `index.md` with its description; add a `log.md` entry for every change.
- Commit messages: imperative subject line, body explaining why.

## Working notes

Practical knowledge learned the hard way. **Keep this section current**: whenever something like this comes up, add it here (or, better, to a script or check) in the same piece of work, and fix or remove notes that turn out to be wrong.

Notes here are public: keep them general. Anything true only of a particular machine goes in `CLAUDE.local.md` (gitignored; Claude Code reads it automatically).

**Windows and networks that intercept HTTPS**

- Where HTTPS is intercepted (antivirus web scanning, company proxies), tools with their own certificate list fail:
  - `uv` needs `--system-certs`, e.g. `uv tool upgrade --system-certs specify-cli`;
  - Node and npm need `NODE_OPTIONS=--use-system-ca`; shells started by an app that was already running don't see a newly saved value, so set it per command;
  - `winget` needs `--source winget`, because the Microsoft Store source pins its certificate;
  - `pip` downloads can arrive corrupted; use `uv` instead.
- Rust lives in `~/.cargo/bin`, which may not be on `PATH` in Claude's shell: `export PATH="$HOME/.cargo/bin:$PATH"`.
- If incremental builds fail with "Access is denied" on their session directories, build with `CARGO_INCREMENTAL=0`.
- Long paths (such as Claude's scratchpad) can exceed the Windows path limit for some tools. Put generated or temporary files under `target/` (gitignored) instead.
- File locks (`File::try_lock`) are mandatory on Windows: while one handle holds the lock, no other handle can read the file, not even SQLite's or `fs::copy` in the same process. Renaming another file over it still works. `uni-store` therefore reads an open project through its locked handle and gives SQLite the bytes (`deserialize`), and locks the new file before renaming it into place on save.
- Python's `open(..., "w")` writes Windows line endings: pass `newline="
"`. When reading `gh` or Python output into shell variables, strip `` (`| tr -d ''`).
- Don't chain checks through `| tail`: it hides the exit code. Check `$?` or `${PIPESTATUS[0]}`, which is how a failing test was once committed and pushed.

**GitHub**

- The bot's token can't run GraphQL queries that read `login`, `name` or `slug` on some objects (it lacks `read:org`). Use REST (`gh api repos/...`) for those.
- The ready gate assigns the owner only when every check passes. Dependabot PRs whose checks fail just sit there; look for them.
- During GitHub incidents (check https://www.githubstatus.com):
  - pushes can be refused with "Internal Server Error": retry later, always naming the branch (`git push origin <branch>:<branch>`), never a bare `git push` from a background job while switching branches;
  - a PR can stop following its branch: closing and reopening it (as the bot) makes GitHub re-read it;
  - "last pusher could not be determined" blocks the merge even with the owner's approval; a fresh push fixes it, or, if the incident is ongoing, the owner may use the admin bypass.

**Spec Kit**

- Each Spec Kit command checks `.specify/extensions.yml` for hooks; there is none yet.
- `.specify/feature.json` points to the current spec folder and is gitignored. Set it when starting a spec.
