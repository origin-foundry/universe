import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_prereqs import Env, format_result, run_checks  # noqa: E402

GOOD = {
    "rustup": "rustup 1.28.2 (e4f3ad6f8 2025-04-28)",
    "cargo": "cargo 1.91.0 (ea2d97820 2025-10-10)",
    "node": "v22.17.0",
    "npm": "10.9.2",
    "vswhere": r"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools",
    "https": "",
}
INSTALLERS = ("winget", "msiexec", "rustup-init", "setup.exe", "vs_buildtools")


def make_env(platform="win32", python=(3, 13, 5), overrides=None, webview="131.0.2903.70"):
    answers = dict(GOOD, **(overrides or {}))
    calls = []

    def run(cmd):
        calls.append(cmd)
        name = Path(cmd[0]).stem.lower()
        if name == "node" and "-e" in cmd:
            name = "https"
        answer = answers.get(name)
        if answer is None:
            return None  # not installed
        if isinstance(answer, tuple):
            return answer  # (returncode, output)
        return 0, answer

    env = Env(platform=platform, run=run, read_webview2=lambda: webview, python_version=python)
    return env, calls


def lines(env):
    return [format_result(r) for r in run_checks(env)]


class AllPresent(unittest.TestCase):
    def test_every_line_ok_and_exit_zero(self):
        env, _ = make_env()
        results = run_checks(env)
        self.assertTrue(results)
        self.assertTrue(all(r.status == "ok" for r in results), [format_result(r) for r in results])
        self.assertFalse(any(r.failed for r in results))


class Missing(unittest.TestCase):
    def test_rust_missing_names_it_and_links_the_docs(self):
        env, _ = make_env(overrides={"rustup": None, "cargo": None})
        out = "\n".join(lines(env))
        self.assertRegex(out, r"(?m)^missing  Rust\b.* - see docs/development\.md#rust$")

    def test_build_tools_missing(self):
        env, _ = make_env(overrides={"vswhere": None})
        self.assertRegex("\n".join(lines(env)), r"(?m)^missing  C\+\+ Build Tools\b.*#c-build-tools$")

    def test_webview2_missing(self):
        env, _ = make_env(webview=None)
        self.assertRegex("\n".join(lines(env)), r"(?m)^missing  WebView2\b.*#webview2$")

    def test_missing_fails_the_check(self):
        env, _ = make_env(overrides={"npm": None})
        self.assertTrue(any(r.failed for r in run_checks(env)))


class TooOld(unittest.TestCase):
    def test_node_20_is_too_old(self):
        env, _ = make_env(overrides={"node": "v20.11.0"})
        out = "\n".join(lines(env))
        self.assertRegex(out, r"(?m)^too old  Node\.js 20\.11\.0 \(need 22 or later\) - see docs/development\.md#nodejs$")

    def test_python_310_is_too_old(self):
        env, _ = make_env(python=(3, 10, 4))
        self.assertRegex("\n".join(lines(env)), r"(?m)^too old  Python 3\.10\.4 \(need 3\.11 or later\).*#python$")


class OtherPlatforms(unittest.TestCase):
    def test_windows_only_items_are_skipped_not_failed(self):
        env, _ = make_env(platform="linux", overrides={"vswhere": None}, webview=None)
        results = {r.name: r for r in run_checks(env)}
        self.assertEqual(results["C++ Build Tools"].status, "skipped")
        self.assertEqual(results["WebView2"].status, "skipped")
        self.assertFalse(any(r.failed for r in results.values()))


class Certificates(unittest.TestCase):
    def test_certificate_error_is_a_warning_not_a_failure(self):
        env, _ = make_env(overrides={"https": (1, "UNABLE_TO_GET_ISSUER_CERT_LOCALLY")})
        results = run_checks(env)
        out = "\n".join(format_result(r) for r in results)
        self.assertRegex(out, r"(?m)^warning  HTTPS certificate error from Node.*#certificate-errors$")
        self.assertFalse(any(r.failed for r in results))

    def test_offline_is_a_warning_without_blaming_certificates(self):
        env, _ = make_env(overrides={"https": (1, "ENOTFOUND")})
        out = "\n".join(lines(env))
        self.assertIn("warning", out)
        self.assertNotIn("certificate error", out)


class NeverInstalls(unittest.TestCase):
    def test_no_installer_is_ever_run(self):
        for overrides in ({}, {"rustup": None, "cargo": None, "node": None, "npm": None, "vswhere": None}):
            env, calls = make_env(overrides=overrides, webview=None)
            run_checks(env)
            for cmd in calls:
                self.assertFalse(any(i in " ".join(cmd).lower() for i in INSTALLERS), cmd)


if __name__ == "__main__":
    unittest.main()
