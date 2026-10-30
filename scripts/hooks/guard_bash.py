#!/usr/bin/env python3
"""Claude Code PreToolUse hook: block shell commands that would bypass the owner's approval steps.

Reads the hook payload on stdin. Exit code 2 blocks the command and shows stderr to Claude.
See the Human Gates section of .specify/memory/constitution.md.
"""
import json
import re
import sys

BLOCKED = [
    (r"\bgit\s+push\b[^;&|]*\s(origin\s+)?(HEAD:)?(refs/heads/)?(main|master)\b", "pushing to main"),
    (r"\bgit\s+push\b[^;&|]*\s(--force\b|-f\b|--force-with-lease\b)", "force-pushing"),
    (r"\bgit\s+push\b[^;&|]*\s--mirror\b", "mirror-pushing"),
    (r"\bgh\s+pr\s+merge\b", "merging a PR"),
    (r"\bgh\s+pr\s+ready\b", "marking a PR ready (CI does this when checks pass)"),
    (r"\bgh\s+pr\s+review\b[^;&|]*--approve\b", "approving a PR"),
    (r"\bgh\s+api\b[^;&|]*/(rulesets|branches/[^/\s]+/protection)\b", "changing repository rules"),
    (r"\bgh\s+(ruleset|repo\s+edit)\b", "changing repository settings"),
    (r"\bgit\s+(commit|push)\b[^;&|]*--no-verify\b", "skipping git hooks"),
]


def main():
    try:
        payload = json.load(sys.stdin)
    except json.JSONDecodeError:
        return 0
    command = (payload.get("tool_input") or {}).get("command") or ""
    for pattern, reason in BLOCKED:
        if re.search(pattern, command):
            print(
                f"Blocked by the repository guard: {reason} is reserved for the owner "
                "(see Human Gates in .specify/memory/constitution.md). "
                "Ask the owner instead of working around this.",
                file=sys.stderr,
            )
            return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
