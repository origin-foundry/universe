import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_pr_policy import PolicyInputs, check  # noqa: E402

FULL_SPEC = {
    "specs/001-foundations/spec.md": "spec",
    "specs/001-foundations/plan.md": "plan",
    "specs/001-foundations/tasks.md": "- [x] T001 one\n- [x] T002 two\n",
}
MERGED_DESIGN = {"merged": True, "labels": ["design"], "files": list(FULL_SPEC)}


def pr(labels, files, body="", repo_files=None, design=None):
    repo_files = FULL_SPEC if repo_files is None else repo_files
    return PolicyInputs(
        labels=labels, body=body, files=files,
        read_file=repo_files.get, read_design_pr=lambda n: design if n == 7 else None,
    )


class KindLabels(unittest.TestCase):
    def test_requires_exactly_one_kind(self):
        self.assertTrue(check(pr([], ["README.md"])))
        self.assertTrue(check(pr(["design", "build"], ["README.md"])))


class DesignPRs(unittest.TestCase):
    def test_complete_spec_passes(self):
        self.assertEqual(check(pr(["design"], list(FULL_SPEC) + ["knowledge/log.md"])), [])

    def test_missing_plan_fails(self):
        files = {k: v for k, v in FULL_SPEC.items() if not k.endswith("plan.md")}
        errors = check(pr(["design"], list(files), repo_files=files))
        self.assertTrue(any("plan.md" in e for e in errors))

    def test_product_code_fails(self):
        errors = check(pr(["design"], list(FULL_SPEC) + ["crates/uni-time/src/lib.rs"]))
        self.assertTrue(any("crates/uni-time" in e for e in errors))

    def test_process_change_passes(self):
        self.assertEqual(check(pr(["design"], [".claude/settings.json", "scripts/hooks/guard_bash.py"])), [])


class BuildPRs(unittest.TestCase):
    def test_linked_merged_design_with_ticked_tasks_passes(self):
        files = ["crates/uni-time/src/lib.rs", "specs/001-foundations/tasks.md"]
        self.assertEqual(check(pr(["build"], files, body="Design: #7", design=MERGED_DESIGN)), [])

    def test_missing_design_reference_fails(self):
        errors = check(pr(["build"], ["crates/x.rs"], design=MERGED_DESIGN))
        self.assertTrue(any("Design: #" in e for e in errors))

    def test_unmerged_design_fails(self):
        design = dict(MERGED_DESIGN, merged=False)
        errors = check(pr(["build"], ["crates/x.rs"], body="Design: #7", design=design))
        self.assertTrue(any("not merged" in e for e in errors))

    def test_unticked_tasks_fail(self):
        files = dict(FULL_SPEC, **{"specs/001-foundations/tasks.md": "- [x] T001\n- [ ] T002\n"})
        errors = check(pr(["build"], ["crates/x.rs"], body="Design: #7", repo_files=files, design=MERGED_DESIGN))
        self.assertTrue(any("1 unticked" in e for e in errors))

    def test_changing_spec_or_knowledge_fails(self):
        for f in ("specs/001-foundations/spec.md", "knowledge/overview.md", ".claude/settings.json",
                  "scripts/check_architecture.py"):
            errors = check(pr(["build"], ["crates/x.rs", f], body="Design: #7", design=MERGED_DESIGN))
            self.assertTrue(any(f in e for e in errors), f)


class FastLanePRs(unittest.TestCase):
    def test_docs_and_ci_pass(self):
        self.assertEqual(check(pr(["fast-lane"], ["README.md", ".github/workflows/ci.yml", "crates/x.rs"])), [])

    def test_guardrails_and_design_files_fail(self):
        for f in (".specify/memory/constitution.md", "scripts/hooks/stop_checks.py",
                  ".github/workflows/pr-policy.yml", ".github/CODEOWNERS", "knowledge/glossary.md",
                  "scripts/check_architecture.py", "scripts/tests/test_check_architecture.py",
                  "scripts/tests/fixtures/architecture/ok/layout.md"):
            errors = check(pr(["fast-lane"], [f]))
            self.assertTrue(errors, f)


if __name__ == "__main__":
    unittest.main()
