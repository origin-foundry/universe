#!/usr/bin/env python3
"""Keep personal and machine-specific details out of this public repository.

Scans the files git tracks, plus new files that aren't ignored (or the paths given), for:

- personal folder paths: `C:\\Users\\<name>`, `/home/<name>`, `/Users/<name>`;
- email addresses, other than GitHub noreply, Anthropic noreply and example addresses;
- any word listed in `.privacy-words`, a gitignored file only the owner keeps (one word or phrase
  per line, `#` for comments). In CI the file doesn't exist, so only the general patterns apply.

A line can opt out with the marker `privacy: allow` (used by this check's own tests).

Usage: python scripts/check_privacy.py [PATH ...]    exit code 1 if anything is found
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORDS_FILE = ROOT / ".privacy-words"
ALLOW_MARKER = "privacy: allow"

USER_PATH = re.compile(
    r"(?:\b[A-Za-z]:[\\/]+Users|(?<![\w.])/home|(?<![\w.])/Users)[\\/]+([^\\/\s\"'`<>()\[\]{},;]+)"
)
GENERIC_USERS = {
    "public", "default", "all users", "runner", "runneradmin", "user", "username", "you", "your-name",
    "yourname", "me", "name", "<name>", "<user>", "%username%", "$user", "${user}", "...", "…", "*",
}
EMAIL = re.compile(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b")
# "icon@2x.png" and similar are file names, not addresses.
FILE_EXTENSIONS = {"png", "jpg", "jpeg", "gif", "svg", "ico", "webp", "icns", "json", "md", "rs", "ts", "tsx", "js"}
ALLOWED_EMAIL = re.compile(
    r"(@users\.noreply\.github\.com|^noreply@anthropic\.com|@example\.(com|org|net)|^git@github\.com)$", re.I
)


def private_words(path=WORDS_FILE):
    if not path.exists():
        return []
    words = []
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line and not line.startswith("#"):
            words.append(line)
    return words


def scan_text(name, text, words=()):
    """Return a list of "file:line: kind: match" findings for one file's text."""
    word_patterns = [(w, re.compile(r"(?<!\w)" + re.escape(w) + r"(?!\w)", re.I)) for w in words]
    findings = []
    for number, line in enumerate(text.splitlines(), start=1):
        if ALLOW_MARKER in line:
            continue
        for m in USER_PATH.finditer(line):
            if m.group(1).lower() not in GENERIC_USERS:
                findings.append(f"{name}:{number}: personal folder path: {m.group(0)}")
        for m in EMAIL.finditer(line):
            if not ALLOWED_EMAIL.search(m.group(0)) and m.group(0).rsplit(".", 1)[-1].lower() not in FILE_EXTENSIONS:
                findings.append(f"{name}:{number}: email address: {m.group(0)}")
        for word, pattern in word_patterns:
            if pattern.search(line):
                findings.append(f"{name}:{number}: private word from .privacy-words: {word}")
    return findings


def tracked_files():
    # Tracked files plus new ones not yet committed (but not gitignored ones).
    out = subprocess.run(["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
                         cwd=ROOT, capture_output=True, text=True, check=True)
    return [ROOT / p for p in out.stdout.split("\0") if p]


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    paths = [Path(p).resolve() for p in argv] if argv else tracked_files()
    words = private_words()
    findings = []
    for path in paths:
        if path.name == ".privacy-words" or not path.is_file():
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue  # binary files, such as icons
        name = path.relative_to(ROOT).as_posix() if path.is_relative_to(ROOT) else str(path)
        findings += scan_text(name, text, words)
    for f in findings:
        print(f"error: {f}")
    extra = f" and {len(words)} private word(s)" if words else ""
    print(f"privacy: {len(findings)} problem(s) in {len(paths)} file(s), general patterns{extra}")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
