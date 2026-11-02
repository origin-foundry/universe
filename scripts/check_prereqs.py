#!/usr/bin/env python3
"""Check that this machine has what's needed to build and run Universe.

Reports each prerequisite as ok, missing, too old, skipped (not needed on this platform) or
warning, with a link to the step in docs/development.md that fixes it. It never installs
anything. Exit code 1 if anything is missing or too old.

Usage: python scripts/check_prereqs.py
See specs/001-app-skeleton/contracts/check-scripts.md.
"""
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Optional

DOCS = "docs/development.md"
MIN_PYTHON = (3, 11)
MIN_NODE = 22
WEBVIEW2_CLIENT = r"Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
MSVC_COMPONENT = "Microsoft.VisualStudio.Component.VC.Tools.x86.x64"
HTTPS_PROBE = (
    "require('https').get('https://registry.npmjs.org/',r=>{r.resume();process.exit(0)})"
    ".on('error',e=>{console.log(e.code||e.message);process.exit(1)})"
    ".setTimeout(8000,function(){console.log('TIMEOUT');this.destroy();process.exit(1)})"
)
CERT_ERRORS = ("CERT", "SELF_SIGNED", "UNABLE_TO_VERIFY", "UNABLE_TO_GET_ISSUER")


@dataclass
class Result:
    status: str  # ok | missing | too old | skipped | warning
    name: str
    detail: str = ""
    anchor: str = ""

    @property
    def failed(self):
        return self.status in ("missing", "too old")


@dataclass
class Env:
    platform: str
    run: Callable[[list], Optional[tuple]]  # None if the program isn't installed, else (code, output)
    read_webview2: Callable[[], Optional[str]]
    python_version: tuple


def format_result(r: Result) -> str:
    text = f"{r.status:<8} {r.name}"
    if r.detail:
        text += f" {r.detail}"
    if r.anchor and r.status in ("missing", "too old", "warning"):
        text += f" - see {DOCS}#{r.anchor}"
    return text


def _version(text, pattern=r"(\d+)\.(\d+)\.(\d+)"):
    m = re.search(pattern, text or "")
    return tuple(int(x) for x in m.groups()) if m else None


def _pinned_rust():
    path = Path(__file__).resolve().parents[1] / "rust-toolchain.toml"
    if path.exists():
        m = re.search(r'channel\s*=\s*"([^"]+)"', path.read_text(encoding="utf-8"))
        if m:
            return m.group(1)
    return None


def check_python(env):
    v = env.python_version
    shown = ".".join(str(x) for x in v[:3])
    if tuple(v[:2]) < MIN_PYTHON:
        return Result("too old", f"Python {shown}", "(need 3.11 or later)", "python")
    return Result("ok", f"Python {shown}")


def check_node(env):
    node = env.run(["node", "--version"])
    if node is None or node[0] != 0:
        return Result("missing", "Node.js", anchor="nodejs")
    v = _version(node[1])
    shown = ".".join(str(x) for x in v) if v else node[1].strip()
    if not v or v[0] < MIN_NODE:
        return Result("too old", f"Node.js {shown}", f"(need {MIN_NODE} or later)", "nodejs")
    return Result("ok", f"Node.js {shown}")


def check_npm(env):
    npm = env.run(["npm", "--version"])
    if npm is None or npm[0] != 0:
        return Result("missing", "npm", "(comes with Node.js)", "nodejs")
    return Result("ok", f"npm {npm[1].strip()}")


def check_rust(env):
    rustup = env.run(["rustup", "--version"])
    cargo = env.run(["cargo", "--version"])
    if rustup is None or rustup[0] != 0 or cargo is None or cargo[0] != 0:
        return Result("missing", "Rust", "(rustup and cargo)", "rust")
    pinned = _pinned_rust()
    note = f"(pinned {pinned}; rustup installs it on first build)" if pinned else "(no version pinned yet)"
    return Result("ok", "Rust", note)


def check_build_tools(env):
    if env.platform != "win32":
        return Result("skipped", "C++ Build Tools", "(Windows only)")
    root = os.environ.get("ProgramFiles(x86)", r"C:\Program Files (x86)")
    vswhere = str(Path(root) / "Microsoft Visual Studio" / "Installer" / "vswhere.exe")
    found = env.run([vswhere, "-products", "*", "-requires", MSVC_COMPONENT, "-property", "installationPath"])
    if found is None or found[0] != 0 or not found[1].strip():
        return Result("missing", "C++ Build Tools", '("Desktop development with C++")', "c-build-tools")
    return Result("ok", "C++ Build Tools", f"({found[1].strip().splitlines()[0]})")


def check_webview2(env):
    if env.platform != "win32":
        return Result("skipped", "WebView2", "(Windows only)")
    version = env.read_webview2()
    if not version or version == "0.0.0.0":
        return Result("missing", "WebView2", anchor="webview2")
    return Result("ok", f"WebView2 {version}")


def check_https(env):
    probe = env.run(["node", "-e", HTTPS_PROBE])
    if probe is None:
        return None  # Node is missing; already reported
    code, output = probe
    if code == 0:
        return Result("ok", "HTTPS from Node")
    if any(marker in output.upper() for marker in CERT_ERRORS):
        return Result("warning", "HTTPS certificate error from Node", f"({output.strip()})", "certificate-errors")
    return Result("warning", "HTTPS from Node", f"(couldn't reach npm: {output.strip() or 'no response'}; offline?)")


def run_checks(env):
    results = [check_python(env), check_node(env), check_npm(env), check_rust(env),
               check_build_tools(env), check_webview2(env)]
    if results[1].status == "ok":
        https = check_https(env)
        if https:
            results.append(https)
    return results


def real_run(cmd):
    exe = cmd[0] if Path(cmd[0]).is_absolute() else shutil.which(cmd[0])
    if not exe or not Path(exe).exists():
        return None
    try:
        p = subprocess.run([exe, *cmd[1:]], capture_output=True, text=True, timeout=30)
    except (OSError, subprocess.TimeoutExpired) as exc:
        return 1, str(exc)
    return p.returncode, (p.stdout + p.stderr).strip()


def real_webview2():
    try:
        import winreg
    except ImportError:
        return None
    places = [
        (winreg.HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\" + WEBVIEW2_CLIENT),
        (winreg.HKEY_LOCAL_MACHINE, "SOFTWARE\\" + WEBVIEW2_CLIENT),
        (winreg.HKEY_CURRENT_USER, "Software\\" + WEBVIEW2_CLIENT),
    ]
    for hive, key in places:
        try:
            with winreg.OpenKey(hive, key) as k:
                return winreg.QueryValueEx(k, "pv")[0]
        except OSError:
            continue
    return None


def main():
    env = Env(platform=sys.platform, run=real_run, read_webview2=real_webview2,
              python_version=tuple(sys.version_info[:3]))
    results = run_checks(env)
    for r in results:
        print(format_result(r))
    failed = [r for r in results if r.failed]
    print(f"{len(results)} checked, {len(failed)} missing or too old")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
