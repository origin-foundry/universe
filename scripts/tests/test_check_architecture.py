import shutil
import subprocess
import sys
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1]
FIXTURES = Path(__file__).resolve().parent / "fixtures" / "architecture"


def check(fixture):
    base = FIXTURES / fixture
    result = subprocess.run(
        [sys.executable, str(SCRIPTS / "check_architecture.py"),
         "--workspace", str(base), "--layout", str(base / "layout.md")],
        capture_output=True, text=True, env={**__import__("os").environ, "GITHUB_ACTIONS": ""},
    )
    return result.returncode, result.stdout + result.stderr


@unittest.skipIf(shutil.which("cargo") is None, "cargo isn't installed; the architecture check needs cargo metadata")
class ArchitectureCheck(unittest.TestCase):
    def assertFinding(self, fixture, pattern, code=1):
        rc, out = check(fixture)
        self.assertEqual(rc, code, out)
        self.assertRegex(out, pattern)

    def test_ok(self):
        rc, out = check("ok")
        self.assertEqual(rc, 0, out)
        self.assertNotIn("error:", out)

    def test_upward_dependency_is_named_as_upward(self):
        self.assertFinding("upward", r"(?m)^error: dependency: low -> high: upward dependency")

    def test_unlisted_edge_points_to_a_design_pr(self):
        self.assertFinding(
            "unlisted",
            r"(?m)^error: dependency: high -> mid: not allowed by .*layout\.md; change the layout in a design PR first$",
        )

    def test_workspace_crate_missing_from_layout(self):
        self.assertFinding("missing-crate", r"(?m)^error: crate: extra: in the workspace but not in the layout")

    def test_layout_crate_absent_from_workspace(self):
        self.assertFinding("absent-crate", r"(?m)^error: crate: ghost: in the layout but not in the workspace")

    def test_cycle_in_layout(self):
        self.assertFinding("cycle", r"(?m)^error: layout: cycle: (a -> b -> a|b -> a -> b)$")

    def test_malformed_layout_block(self):
        self.assertFinding("bad-block", r"(?m)^error: layout: crate-dependencies block isn't valid YAML")

    def test_float_type_in_exact_crate(self):
        self.assertFinding("float-type", r"(?m)^error: float: crates/mid/src/lib\.rs:2: f64 in exact crate mid$")

    def test_float_literal_in_exact_crate(self):
        self.assertFinding("float-literal", r"(?m)^error: float: crates/mid/src/lib\.rs:2: 2\.5e3 in exact crate mid$")

    def test_floats_in_comments_strings_tuples_and_ranges_are_ignored(self):
        rc, out = check("float-ignored")
        self.assertEqual(rc, 0, out)

    def test_unreadable_layout_cannot_run(self):
        base = FIXTURES / "ok"
        result = subprocess.run(
            [sys.executable, str(SCRIPTS / "check_architecture.py"),
             "--workspace", str(base), "--layout", str(base / "missing.md")],
            capture_output=True, text=True,
        )
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
