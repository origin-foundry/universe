#!/usr/bin/env python3
"""Enforce the spec-driven two-PR workflow (ADR 0006) on a pull request.

Every PR carries exactly one kind label:

  design     Spec, plan, tasks, knowledge and process changes. No product code.
  build      Product code implementing a merged design PR named in the body as `Design: #N`.
             May only tick tasks in specs/; every task in the linked spec must be ticked.
  fast-lane  No behaviour change. Must not touch design or guardrail files.

Guardrail files (constitution, Claude Code settings and hooks, this policy, CODEOWNERS) can only
change in a design PR, so loosening a guardrail always gets design-level review.

In CI this runs from the base branch under `pull_request_target` and reads the PR through the
GitHub API; it never executes code from the PR. `PolicyInputs` keeps it testable offline.
"""
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass, field
from fnmatch import fnmatch

KINDS = ("design", "build", "fast-lane")

DESIGN_ALLOWED = [
    "specs/*", "knowledge/*", ".specify/*", ".github/*", ".claude/*", "scripts/*",
    "CLAUDE.md", "README.md", "LICENSE", ".gitignore", ".gitattributes", "DESIGN.md",
]
PRODUCT_CODE = ["crates/*", "ui/*", "src-tauri/*", "Cargo.toml", "Cargo.lock", "package.json", "package-lock.json"]
GUARDRAILS = [
    ".specify/memory/constitution.md", ".claude/*", "scripts/hooks/*", "scripts/check_pr_policy.py",
    ".github/CODEOWNERS", ".github/workflows/pr-policy.yml", ".github/workflows/ready-gate.yml",
]
DESIGN_ONLY = ["knowledge/*", "specs/*"] + GUARDRAILS
SPEC_DIR_RE = re.compile(r"^specs/(\d{3}-[a-z0-9-]+)/")
DESIGN_REF_RE = re.compile(r"^Design:\s*#(\d+)\s*$", re.M | re.I)
UNCHECKED_RE = re.compile(r"^\s*- \[ \]", re.M)


def matches(path, patterns):
    return any(fnmatch(path, p) for p in patterns)


@dataclass
class PolicyInputs:
    labels: list
    body: str
    files: list
    read_file: callable = None          # (path) -> str | None, at the PR head
    read_design_pr: callable = None     # (number) -> {"merged": bool, "labels": [...], "files": [...]} | None
    errors: list = field(default_factory=list)


def check(pr: PolicyInputs):
    errors = []
    kinds = [k for k in KINDS if k in pr.labels]
    if len(kinds) != 1:
        return [f"PR must carry exactly one of the labels {', '.join(KINDS)} (found: {', '.join(kinds) or 'none'})"]
    kind = kinds[0]

    if kind == "design":
        for f in pr.files:
            if not matches(f, DESIGN_ALLOWED) or matches(f, PRODUCT_CODE):
                errors.append(f"design PR changes product code or an unexpected file: {f}")
        for spec in sorted({m.group(1) for f in pr.files if (m := SPEC_DIR_RE.match(f))}):
            for required in ("spec.md", "plan.md", "tasks.md"):
                if pr.read_file(f"specs/{spec}/{required}") is None:
                    errors.append(f"specs/{spec}/ is missing {required}")

    elif kind == "build":
        for f in pr.files:
            if matches(f, DESIGN_ONLY) and not re.match(r"^specs/[^/]+/tasks\.md$", f):
                errors.append(f"build PR changes a design or guardrail file (needs a design PR): {f}")
        ref = DESIGN_REF_RE.search(pr.body or "")
        if not ref:
            errors.append("build PR body must contain a line `Design: #<design PR number>`")
        else:
            design = pr.read_design_pr(int(ref.group(1)))
            if design is None:
                errors.append(f"design PR #{ref.group(1)} was not found")
            else:
                if not design["merged"]:
                    errors.append(f"design PR #{ref.group(1)} is not merged")
                if "design" not in design["labels"]:
                    errors.append(f"PR #{ref.group(1)} is not labelled design")
                specs = sorted({m.group(1) for f in design["files"] if (m := SPEC_DIR_RE.match(f))})
                if not specs:
                    errors.append(f"design PR #{ref.group(1)} contains no spec folder")
                for spec in specs:
                    tasks = pr.read_file(f"specs/{spec}/tasks.md")
                    if tasks is None:
                        errors.append(f"specs/{spec}/tasks.md not found")
                    elif n := len(UNCHECKED_RE.findall(tasks)):
                        errors.append(f"specs/{spec}/tasks.md has {n} unticked task(s)")

    elif kind == "fast-lane":
        for f in pr.files:
            if matches(f, DESIGN_ONLY):
                errors.append(f"fast-lane PR changes a design or guardrail file (needs a design PR): {f}")

    return errors


def gh_json(*args):
    out = subprocess.run(["gh", *args], capture_output=True, text=True)
    return json.loads(out.stdout) if out.returncode == 0 and out.stdout.strip() else None


def from_github(repo, number):
    pr = gh_json("pr", "view", str(number), "--repo", repo, "--json", "labels,body,files,headRefOid")
    head = pr["headRefOid"]

    def read_file(path):
        out = subprocess.run(
            ["gh", "api", f"repos/{repo}/contents/{path}?ref={head}", "-H", "Accept: application/vnd.github.raw"],
            capture_output=True, text=True,
        )
        return out.stdout if out.returncode == 0 else None

    def read_design_pr(n):
        d = gh_json("pr", "view", str(n), "--repo", repo, "--json", "mergedAt,labels,files")
        if d is None:
            return None
        return {"merged": bool(d["mergedAt"]), "labels": [l["name"] for l in d["labels"]],
                "files": [f["path"] for f in d["files"]]}

    return PolicyInputs(
        labels=[l["name"] for l in pr["labels"]], body=pr["body"] or "",
        files=[f["path"] for f in pr["files"]], read_file=read_file, read_design_pr=read_design_pr,
    )


def main():
    repo, number = os.environ["GITHUB_REPOSITORY"], os.environ["PR_NUMBER"]
    errors = check(from_github(repo, number))
    for e in errors:
        print(f"::error::{e}")
    print("PR policy: " + ("passed" if not errors else f"{len(errors)} problem(s)"))
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
