#!/usr/bin/env python3
"""Enforce the crate layout (constitution principles III and V).

The allowed dependencies between workspace crates, and the crates that must stay exact, are
read from the `crate-dependencies` block in knowledge/architecture/crate-layout.md, the single
source of truth (spec 001, FR-012a). This check fails when:

- the block is missing, malformed or contains a cycle;
- the workspace and the block disagree about which crates exist;
- a workspace crate depends on another in a way the block doesn't allow;
- an exact crate uses a floating-point type or literal (comments and strings are ignored).

Usage: python scripts/check_architecture.py [--workspace DIR] [--layout FILE]
Exit codes: 0 all rules hold, 1 findings, 2 can't run. See
specs/001-app-skeleton/contracts/check-scripts.md.
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
BLOCK_RE = re.compile(r"^```yaml crate-dependencies[ \t]*\n(.*?)^```", re.M | re.S)
FLOAT_RES = [
    re.compile(r"\bf(?:32|64)\b"),
    re.compile(r"(?<![\w.])\d[\d_]*(?:f32|f64)\b"),
    re.compile(r"(?<![\w.])\d[\d_]*\.\d[\d_]*(?:[eE][+-]?\d[\d_]*)?(?:f32|f64)?\b"),
    re.compile(r"(?<![\w.])\d[\d_]*[eE][+-]?\d[\d_]*\b"),
    re.compile(r"(?<![\w.])\d[\d_]*\.(?![\d.\w])"),
]


class CannotRun(Exception):
    pass


def read_layout(path):
    """Return (allowed, exact, findings) from the layout file's crate-dependencies block."""
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as exc:
        raise CannotRun(f"can't read {path}: {exc}") from exc
    blocks = BLOCK_RE.findall(text)
    if len(blocks) != 1:
        return None, None, [f"layout: expected exactly one crate-dependencies block, found {len(blocks)}"]
    try:
        data = yaml.safe_load(blocks[0])
    except yaml.YAMLError as exc:
        reason = str(exc).splitlines()[0]
        return None, None, [f"layout: crate-dependencies block isn't valid YAML ({reason})"]
    if not isinstance(data, dict):
        return None, None, ["layout: crate-dependencies block must be a mapping of crate to list"]

    exact = data.pop("exact", [])
    findings = []
    for name, deps in data.items():
        if not isinstance(deps, list) or not all(isinstance(d, str) for d in deps):
            findings.append(f"layout: {name}: must map to a list of crate names")
            continue
        for dep in deps:
            if dep == name:
                findings.append(f"layout: {name}: depends on itself")
            elif dep not in data:
                findings.append(f"layout: {name}: depends on {dep}, which isn't listed")
    for name in exact or []:
        if name not in data:
            findings.append(f"layout: exact: {name} isn't listed")
    if findings:
        return None, None, findings

    cycle = find_cycle(data)
    if cycle:
        return None, None, ["layout: cycle: " + " -> ".join(cycle)]
    return data, set(exact or []), []


def find_cycle(graph):
    state = {}

    def visit(node, path):
        state[node] = "visiting"
        for dep in graph[node]:
            if state.get(dep) == "visiting":
                return path[path.index(dep):] + [dep]
            if dep not in state:
                found = visit(dep, path + [dep])
                if found:
                    return found
        state[node] = "done"
        return None

    for node in sorted(graph):
        if node not in state:
            found = visit(node, [node])
            if found:
                return found
    return None


def reachable(graph, start, target):
    seen, stack = set(), [start]
    while stack:
        node = stack.pop()
        if node == target:
            return True
        if node not in seen:
            seen.add(node)
            stack.extend(graph.get(node, []))
    return False


def workspace_crates(workspace):
    """Return {crate: (directory, {workspace dependencies})} from cargo metadata."""
    cargo = shutil.which("cargo")
    if cargo is None:
        raise CannotRun("cargo isn't installed")
    result = subprocess.run(
        [cargo, "metadata", "--format-version", "1", "--no-deps",
         "--manifest-path", str(workspace / "Cargo.toml")],
        capture_output=True, text=True,
    )
    if result.returncode != 0:
        raise CannotRun("cargo metadata failed: " + result.stderr.strip().splitlines()[-1])
    packages = json.loads(result.stdout)["packages"]
    names = {p["name"] for p in packages}
    return {
        p["name"]: (Path(p["manifest_path"]).parent,
                    {d["name"] for d in p["dependencies"] if d["name"] in names and d.get("path")})
        for p in packages
    }


def check_dependencies(allowed, crates, layout_name):
    findings = []
    for name in sorted(set(crates) - set(allowed)):
        findings.append(f"crate: {name}: in the workspace but not in the layout")
    for name in sorted(set(allowed) - set(crates)):
        findings.append(f"crate: {name}: in the layout but not in the workspace")
    for name, (_, deps) in sorted(crates.items()):
        for dep in sorted(deps):
            if name not in allowed or dep in allowed.get(name, []):
                continue
            if dep in allowed and reachable(allowed, dep, name):
                findings.append(f"dependency: {name} -> {dep}: upward dependency ({dep} is above {name} "
                                f"in {layout_name}); change the layout in a design PR first")
            else:
                findings.append(f"dependency: {name} -> {dep}: not allowed by {layout_name}; "
                                f"change the layout in a design PR first")
    return findings


def strip_comments_and_strings(source):
    """Blank out comments and string/char literals, keeping line breaks so line numbers hold."""
    out, i, n = [], 0, len(source)

    def blank(text):
        return re.sub(r"[^\n]", " ", text)

    while i < n:
        ch = source[i]
        if source.startswith("//", i):
            end = source.find("\n", i)
            end = n if end == -1 else end
            out.append(blank(source[i:end]))
            i = end
        elif source.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if source.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif source.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            out.append(blank(source[i:j]))
            i = j
        elif (m := re.match(r'(?:b|c)?r(#*)"', source[i:])) and (i == 0 or not (source[i - 1].isalnum() or source[i - 1] == "_")):
            end = source.find('"' + m.group(1), i + m.end())
            end = n if end == -1 else end + 1 + len(m.group(1))
            out.append(blank(source[i:end]))
            i = end
        elif ch == '"':
            j = i + 1
            while j < n and source[j] != '"':
                j += 2 if source[j] == "\\" else 1
            out.append(blank(source[i:j + 1]))
            i = j + 1
        elif ch == "'" and (m := re.match(r"'(?:\\.[^']*|[^\\'\n])'", source[i:])):
            out.append(blank(m.group(0)))  # a char literal, not a lifetime
            i += m.end()
        else:
            out.append(ch)
            i += 1
    return "".join(out)


def check_floats(crates, exact, workspace):
    findings = []
    for name in sorted(exact & set(crates)):
        directory = crates[name][0]
        for path in sorted(directory.rglob("*.rs")):
            if "target" in path.relative_to(directory).parts:
                continue
            code = strip_comments_and_strings(path.read_text(encoding="utf-8"))
            for number, line in enumerate(code.splitlines(), start=1):
                hits = sorted({(m.start(), m.group(0)) for rx in FLOAT_RES for m in rx.finditer(line)})
                for _, token in hits[:1]:
                    shown = path.relative_to(workspace).as_posix()
                    findings.append(f"float: {shown}:{number}: {token} in exact crate {name}")
    return findings


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--workspace", type=Path, default=ROOT)
    parser.add_argument("--layout", type=Path, default=ROOT / "knowledge" / "architecture" / "crate-layout.md")
    args = parser.parse_args()
    workspace = args.workspace.resolve()
    layout_name = (args.layout.as_posix() if not args.layout.is_absolute()
                   else os.path.relpath(args.layout, workspace).replace(os.sep, "/"))
    if layout_name.startswith(".."):
        layout_name = args.layout.name

    try:
        allowed, exact, findings = read_layout(args.layout)
        if not findings:
            crates = workspace_crates(workspace)
            findings = check_dependencies(allowed, crates, layout_name) + check_floats(crates, exact, workspace)
    except CannotRun as exc:
        print(f"error: can't run the architecture check: {exc}", file=sys.stderr)
        return 2

    prefix = "::error::" if os.environ.get("GITHUB_ACTIONS") == "true" else ""
    for finding in findings:
        print(f"{prefix}error: {finding}")
    print(f"architecture: {len(findings)} problem(s)" if findings else "architecture: all rules hold")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
