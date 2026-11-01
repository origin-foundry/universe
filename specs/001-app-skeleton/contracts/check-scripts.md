# Contract: repository check scripts

Both scripts use only the Python standard library (3.11+), apart from PyYAML for the architecture check, which CI already installs. Each prints one line per finding and a summary line.

## `scripts/check_architecture.py`

**Usage**: `python scripts/check_architecture.py [--workspace DIR] [--layout FILE]`. The defaults are the repository root and `knowledge/architecture/crate-layout.md`. Fixtures in the tests pass their own paths.

**Exit codes**: 0 = all rules hold; 1 = one or more findings; 2 = can't run (for example, `cargo` isn't installed, or the layout file can't be read).

**Finding format** (one per line, prefixed for GitHub annotations in CI):

```
error: layout: <message>                          # problem in the crate-dependencies block
error: dependency: <crate> -> <crate>: <message>  # disallowed or unlisted edge
error: crate: <crate>: <message>                  # crate missing from the list, or listed but absent
error: float: <path>:<line>: <token> in exact crate <crate>
```

**Messages for unlisted edges** say: "not allowed by knowledge/architecture/crate-layout.md; change the layout in a design PR first".

## `scripts/check_prereqs.py`

**Usage**: `python scripts/check_prereqs.py`. Run automatically before `npm run dev` and `npm run app`.

**Exit codes**: 0 = everything present; 1 = something missing or too old. It never installs anything.

**Output**: one line per prerequisite:

```
ok       Rust 1.xx.x (pinned 1.xx.x, target x86_64-pc-windows-msvc)
missing  Microsoft C++ Build Tools - see docs/development.md#c-build-tools
too old  Node.js 20.11.0 (need 22 or later) - see docs/development.md#nodejs
warning  HTTPS certificate error from Node - see docs/development.md#certificate-errors
```

On Linux and macOS it skips the Windows-only items (the build tools and WebView2) and says so, so it's still useful in CI.
