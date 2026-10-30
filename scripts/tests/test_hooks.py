import json
import subprocess
import sys
import unittest
from pathlib import Path

HOOKS = Path(__file__).resolve().parents[1] / "hooks"


def guard(command):
    payload = json.dumps({"tool_name": "Bash", "tool_input": {"command": command}})
    return subprocess.run(
        [sys.executable, str(HOOKS / "guard_bash.py")], input=payload, capture_output=True, text=True
    ).returncode


class GuardBlocks(unittest.TestCase):
    def test_blocked(self):
        for command in [
            "git push origin main",
            "git push -u origin HEAD:main",
            "git push origin refs/heads/main",
            "git push --force origin setup/x",
            "git push -f",
            "git push --force-with-lease origin feature",
            "gh pr merge 12 --squash",
            "gh pr ready 12",
            "gh pr review 12 --approve",
            "gh api -X PUT repos/o/r/rulesets/1",
            "gh api repos/o/r/branches/main/protection",
            "gh repo edit --visibility private",
            "git commit -m x --no-verify",
            "cd x && git push origin main",
        ]:
            self.assertEqual(guard(command), 2, command)

    def test_allowed(self):
        for command in [
            "git push -u origin setup/workflow-guardrails",
            "git push origin feature/maintenance-window",
            "git switch main",
            "gh pr create --draft --label design",
            "gh pr view 12",
            "gh pr review 12 --comment -b ok",
            "python scripts/check_okf.py knowledge",
        ]:
            self.assertEqual(guard(command), 0, command)

    def test_bad_payload_is_allowed(self):
        result = subprocess.run([sys.executable, str(HOOKS / "guard_bash.py")], input="not json",
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
