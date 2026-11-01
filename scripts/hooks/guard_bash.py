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
    (r"\bgh\s+pr\s+ready\b", "changing a PR's draft state (PRs open ready for review)"),
    (r"\bgh\s+pr\s+review\b[^;&|]*--approve\b", "approving a PR"),
    (r"\bgh\s+api\b[^;&|]*/(rulesets|branches/[^/\s]+/protection)\b", "changing repository rules"),
    (r"\bgh\s+(ruleset|repo\s+edit)\b", "changing repository settings"),
    (r"\bgit\s+(commit|push)\b[^;&|]*--no-verify\b", "skipping git hooks"),
    # The bot token is never read, printed or bypassed, and Claude never acts as the owner.
    (r"\benv\b[^;&|]*\s-u\s*GH_TOKEN\b", "dropping the bot token to act as the owner"),
    (r"\bunset\s+GH_TOKEN\b|Remove-Item\s+Env:\\?GH_TOKEN", "dropping the bot token to act as the owner"),
    (r"(^|[\s;&|(])GH_TOKEN=|\$env:GH_TOKEN\s*=", "overriding the bot token"),
    (r"\bgh\s+auth\s+(token|switch|login|logout)\b", "reading or switching GitHub credentials"),
]

# Commands may mention the token file only to confirm it is gitignored.
TOKEN_FILE = re.compile(r"settings\.local\.json")
TOKEN_FILE_ALLOWED = re.compile(r"^\s*git\s+check-ignore\b[^;&|]*$")


def main():
    try:
        payload = json.load(sys.stdin)
    except json.JSONDecodeError:
        return 0
    command = (payload.get("tool_input") or {}).get("command") or ""
    reasons = [reason for pattern, reason in BLOCKED if re.search(pattern, command)]
    if TOKEN_FILE.search(command) and not TOKEN_FILE_ALLOWED.match(command):
        reasons.append("touching the bot token file")
    if not reasons:
        return 0
    print(
        f"Blocked by the repository guard: {reasons[0]} is reserved for the owner "
        "(see Human Gates in .specify/memory/constitution.md). "
        "Ask the owner instead of working around this.",
        file=sys.stderr,
    )
    return 2


if __name__ == "__main__":
    sys.exit(main())
