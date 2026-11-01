#!/usr/bin/env python3
"""Claude Code Stop hook: run the fast local checks before Claude ends a turn.

Exit code 2 sends the failures back to Claude so it keeps working. To avoid an endless loop,
if Claude is already continuing because of this hook and the checks still fail, the hook lets
the turn end and warns the owner instead.

Checks only run for parts of the repo that exist, so it is safe before any Rust or UI code.
"""
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def run(name, args, cwd=ROOT):
    exe = shutil.which(args[0])
    if exe is None:
        return f"{name}: `{args[0]}` is not installed, so this check could not run"
    result = subprocess.run([exe, *args[1:]], cwd=cwd, capture_output=True, text=True)
    if result.returncode == 0:
        return None
    output = (result.stdout + result.stderr).strip().splitlines()
    return f"{name} failed:\n" + "\n".join(output[-30:])


def has_changes():
    result = subprocess.run(
        ["git", "status", "--porcelain"], cwd=ROOT, capture_output=True, text=True
    )
    return result.returncode != 0 or bool(result.stdout.strip())


def main():
    try:
        payload = json.load(sys.stdin)
    except json.JSONDecodeError:
        payload = {}
    if not has_changes():
        return 0

    failures = [
        run("Knowledge bundle", [sys.executable, "scripts/check_okf.py", "knowledge"]),
        run("Script tests", [sys.executable, "-m", "unittest", "discover", "-s", "scripts/tests"]),
    ]
    if (ROOT / "Cargo.toml").exists():
        failures.append(run("cargo fmt", ["cargo", "fmt", "--all", "--check"]))
        failures.append(
            run("cargo clippy", ["cargo", "clippy", "--workspace", "--all-targets", "--quiet", "--", "-D", "warnings"])
        )
    if (ROOT / "ui" / "package.json").exists():
        failures.append(run("UI typecheck", ["npm", "run", "--silent", "typecheck"], cwd=ROOT / "ui"))
        failures.append(run("UI lint", ["npm", "run", "--silent", "lint"], cwd=ROOT / "ui"))
    failures = [f for f in failures if f]
    if not failures:
        return 0

    report = "\n\n".join(failures)
    if payload.get("stop_hook_active"):
        print(json.dumps({"systemMessage": "Local checks are still failing after a retry:\n\n" + report}))
        return 0
    print("Local checks failed. Fix these before finishing:\n\n" + report, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
