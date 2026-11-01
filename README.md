# Universe

A desktop app for defining physically grounded, non-Earth calendars across multiple star systems, and converting dates between them and between observers whose clocks run at different rates.

Status: Phase 0 (foundations). The app skeleton exists: an empty window, the Rust workspace, and the checks that guard every change. No calendar or physics features yet.

- **Design:** [knowledge/](knowledge/index.md), an [Open Knowledge Format](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md) bundle.
- **Rules:** [constitution](.specify/memory/constitution.md).
- **How changes are made:** [the feature workflow](knowledge/process/feature-workflow.md), from [ADR 0006](knowledge/decisions/0006-spec-driven-two-pr-workflow.md) and [ADR 0007](knowledge/decisions/0007-bot-identity-and-gates.md).

## Development

Windows 10 or 11. [docs/development.md](docs/development.md) has the step-by-step setup, then:

- `npm ci` once, to install the UI's dependencies
- `npm run dev` to run the app with live reload
- `npm run app` to build the optimised app and run it

A personal project developed in the open. Licensed under GPL-3.0.
