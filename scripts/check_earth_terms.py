#!/usr/bin/env python3
"""Keep the code free of Earth-specific calendar words (ADR 0009).

The engine models calendars in general, so its code names things with the model's own terms
(the era's unit, member, fundamental unit, subdivision) rather than Earth's (year, month, day,
hour). Earth words are allowed only where they really are about Earth:

- in files whose path contains "gregorian" (the Gregorian-shaped test calendar and its fixture);
- on a line that also says "Earth" or "Gregorian", such as an aside in a comment
  ("the era's unit; on Earth, the year");
- between a line containing `earth-terms: off` and one containing `earth-terms: on`.

Words are found inside identifiers too: `year_def`, `HUNDRED_THOUSAND_YEARS` and `LeapRuleDef`
are all caught. "Second" is not an Earth word: it is the SI unit physics uses.

Scans the code git tracks (crates/, ui/src/, src-tauri/), or the paths given.

Usage: python scripts/check_earth_terms.py [PATH ...]    exit code 1 if anything is found
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CODE_DIRS = ("crates/", "ui/src/", "src-tauri/")
CODE_SUFFIXES = {".rs", ".ts", ".tsx", ".js", ".mjs", ".py"}

EARTH_WORDS = {
    "year", "years", "yearly", "annual", "annually",
    "month", "months", "monthly",
    "week", "weeks", "weekly", "weekday", "weekdays", "fortnight", "fortnights",
    "day", "days", "daily", "midnight",
    "hour", "hours", "hourly",
    "minute", "minutes",
    "decade", "decades", "century", "centuries", "millennium", "millennia",
    "leap",
    "january", "february", "april", "june", "july", "september", "october", "november",
    "december",
    "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday",
}
ALLOW_LINE = re.compile(r"earth|gregorian", re.I)
OFF_MARKER = "earth-terms: off"
ON_MARKER = "earth-terms: on"
WORD = re.compile(r"[A-Za-z]+")
CAMEL_PART = re.compile(r"[A-Z]+(?![a-z])|[A-Z]?[a-z]+")


def words(line):
    """Every word in the line, with identifiers split at underscores and case changes."""
    for token in WORD.findall(line):
        for part in CAMEL_PART.findall(token):
            yield part.lower()


def scan_text(name, text):
    """Return a list of "file:line: Earth word: <word>" findings for one file's text."""
    if "gregorian" in name.lower():
        return []
    findings = []
    off = False
    for number, line in enumerate(text.splitlines(), start=1):
        has_off, has_on = OFF_MARKER in line, ON_MARKER in line
        if has_off != has_on:  # a line naming both markers only mentions them
            off = has_off
            continue
        if off or ALLOW_LINE.search(line):
            continue
        found = sorted({w for w in words(line) if w in EARTH_WORDS})
        if found:
            findings.append(f"{name}:{number}: Earth word: {', '.join(found)}")
    if off:
        findings.append(f"{name}: `{OFF_MARKER}` is never followed by `{ON_MARKER}`")
    return findings


def code_files():
    out = subprocess.run(["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
                         cwd=ROOT, capture_output=True, text=True, check=True)
    return [ROOT / p for p in out.stdout.split("\0")
            if p.startswith(CODE_DIRS) and Path(p).suffix in CODE_SUFFIXES]


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    paths = [Path(p).resolve() for p in argv] if argv else code_files()
    findings = []
    for path in paths:
        if not path.is_file():
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        name = path.relative_to(ROOT).as_posix() if path.is_relative_to(ROOT) else str(path)
        findings += scan_text(name, text)
    for f in findings:
        print(f"error: {f}")
    print(f"earth terms: {len(findings)} problem(s) in {len(paths)} file(s)")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
