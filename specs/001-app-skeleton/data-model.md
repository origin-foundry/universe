# Data Model: App Skeleton

This feature stores no user data. Its "data" is the description of the codebase that the architecture check enforces.

## Component (a workspace crate)

| Field | Meaning | Rule |
|---|---|---|
| `name` | Crate name, e.g. `uni-calendar` | Must appear exactly once in the crate-dependencies list |
| `path` | `crates/<name>/` | Every crate lives under `crates/` |
| `exact` | Whether floating point is forbidden | True for `uni-calendar`, `uni-expr`, `uni-format`, `uni-zones` |
| `dependencies` | Other workspace crates it depends on (normal, build and dev) | Each must be in its allowed list |

## Crate-dependencies list

A fenced YAML block in `knowledge/architecture/crate-layout.md`, opened with ```` ```yaml crate-dependencies ````.

```yaml crate-dependencies
uni-time: []
rebound-sys: []
uni-expr: [uni-time]
uni-zones: [uni-time, uni-expr]
uni-calendar: [uni-time, uni-expr, uni-zones]
uni-format: [uni-time, uni-expr, uni-zones, uni-calendar]
uni-physics: [uni-time, uni-expr, rebound-sys]
uni-store: [uni-time, uni-expr, uni-zones, uni-calendar, uni-format, uni-physics]
uni-presets: [uni-time, uni-expr, uni-zones, uni-calendar, uni-format, uni-physics]
uni-app: [uni-time, uni-expr, uni-zones, uni-calendar, uni-format, uni-physics, uni-store, uni-presets]
exact: [uni-calendar, uni-expr, uni-format, uni-zones]
```

**Validation** (by the architecture check):

- The block exists exactly once and parses as a mapping. Every value is a list of crate names, except for the `exact` key.
- Every name in a list is itself a key, and no crate depends on itself.
- The graph has no cycles. "Below" means "reachable through allowed edges", so an acyclic list is the downward-only rule.
- The keys (other than `exact`) are exactly the workspace's crates.
- Every crate named in `exact` is a key.

**Changing it**: through a design PR that edits the knowledge concept. The owner then re-verifies the concept (constitution principle II).

## Check result

| Field | Meaning |
|---|---|
| `check` | The job name the ruleset requires, e.g. `architecture` |
| `platform` | `ubuntu-latest` or `windows-latest`, where it matters |
| `result` | passed, failed or skipped |
| `findings` | For failures: the rule, plus the crate, `file:line` or platform involved |
