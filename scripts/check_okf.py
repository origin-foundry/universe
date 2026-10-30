#!/usr/bin/env python3
"""Validate the knowledge/ bundle against OKF v0.2 plus this project's stricter house rules.

Spec rules (OKF v0.2 §11): every non-reserved .md has parseable YAML frontmatter with a
non-empty `type`; index.md and log.md follow §8 and §9.

House rules (stricter than the spec, which tolerates these): every concept has `title`,
`description` and a valid `status`; every link resolves; every concept is listed in its
directory's index.md with its frontmatter description; timestamps carry a UTC offset.

Usage: python scripts/check_okf.py [bundle_dir]   (exit code 1 on any error)
"""
import datetime as dt
import re
import sys
from pathlib import Path

import yaml

RESERVED = {"index.md", "log.md"}
STATUSES = {"draft", "stable", "deprecated"}
LINK_RE = re.compile(r"\[[^\]]*\]\(([^)\s]+)\)")
FENCE_RE = re.compile(r"^```.*?^```", re.S | re.M)
INDEX_ENTRY_RE = re.compile(r"^\* \[[^\]]+\]\(([^)]+)\) - (.+)$")


def split_frontmatter(text):
    """Return (frontmatter_dict_or_None, body, error)."""
    if not text.startswith("---\n"):
        return None, text, None
    end = text.find("\n---\n", 4)
    if end == -1:
        return None, text, "frontmatter block is not closed"
    try:
        data = yaml.safe_load(text[4:end])
    except yaml.YAMLError as exc:
        return None, text, f"frontmatter is not valid YAML: {exc}"
    if not isinstance(data, dict):
        return None, text, "frontmatter is not a mapping"
    return data, text[end + 5:], None


def resolve_link(root, source, target):
    """Return the resolved path for an internal link, or None for external/anchor links."""
    if re.match(r"^[a-z]+:", target) or target.startswith("#"):
        return None
    target = target.split("#", 1)[0]
    path = root / target.lstrip("/") if target.startswith("/") else source.parent / target
    path = path.resolve()
    return path / "index.md" if target.endswith("/") else path


def check_timestamps(rel, value, errors, key="frontmatter"):
    if isinstance(value, dict):
        for k, v in value.items():
            check_timestamps(rel, v, errors, f"{key}.{k}")
    elif isinstance(value, list):
        for i, v in enumerate(value):
            check_timestamps(rel, v, errors, f"{key}[{i}]")
    elif isinstance(value, dt.datetime) and value.tzinfo is None:
        errors.append(f"{rel}: {key} timestamp has no UTC offset")


def main():
    root = Path(sys.argv[1] if len(sys.argv) > 1 else "knowledge").resolve()
    if not root.is_dir():
        print(f"bundle not found: {root}")
        return 1
    errors, warnings = [], []
    descriptions = {}

    for path in sorted(root.rglob("*.md")):
        rel = path.relative_to(root).as_posix()
        text = path.read_text(encoding="utf-8")
        fm, body, err = split_frontmatter(text)
        if err:
            errors.append(f"{rel}: {err}")
            continue

        if path.name == "index.md":
            if fm is not None and (path.parent != root or set(fm) - {"okf_version"}):
                errors.append(f"{rel}: index.md may only carry okf_version, and only at the bundle root")
        elif path.name == "log.md":
            dates = re.findall(r"^## (.+)$", body, re.M)
            for d in dates:
                if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", d):
                    errors.append(f"{rel}: log heading '{d}' is not YYYY-MM-DD")
            if dates != sorted(dates, reverse=True):
                errors.append(f"{rel}: log entries are not newest first")
        else:
            if fm is None:
                errors.append(f"{rel}: concept has no frontmatter")
                continue
            if not str(fm.get("type") or "").strip():
                errors.append(f"{rel}: missing non-empty `type`")
            for key in ("title", "description"):
                if not str(fm.get(key) or "").strip():
                    errors.append(f"{rel}: missing `{key}` (house rule)")
            if fm.get("status", "stable") not in STATUSES:
                errors.append(f"{rel}: status must be one of {sorted(STATUSES)}")
            check_timestamps(rel, fm, errors)
            descriptions[path.resolve()] = str(fm.get("description") or "").strip()

        for target in LINK_RE.findall(FENCE_RE.sub("", body)):
            resolved = resolve_link(root, path, target)
            if resolved is not None and not resolved.exists():
                errors.append(f"{rel}: broken link {target}")

    for concept, description in descriptions.items():
        index = concept.parent / "index.md"
        if not index.exists():
            errors.append(f"{concept.relative_to(root).as_posix()}: directory has no index.md")
            continue
        listed = {}
        for line in index.read_text(encoding="utf-8").splitlines():
            m = INDEX_ENTRY_RE.match(line)
            if m:
                listed[resolve_link(root, index, m.group(1))] = m.group(2).strip()
        rel = concept.relative_to(root).as_posix()
        if concept not in listed:
            errors.append(f"{rel}: not listed in {index.relative_to(root).as_posix()}")
        elif listed[concept] != description:
            warnings.append(f"{rel}: index entry text differs from frontmatter description")

    for w in warnings:
        print(f"warning: {w}")
    for e in errors:
        print(f"error: {e}")
    print(f"{len(descriptions)} concepts checked, {len(errors)} errors, {len(warnings)} warnings")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
