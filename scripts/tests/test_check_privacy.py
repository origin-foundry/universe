import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_privacy import private_words, scan_text  # noqa: E402

# Every example below is on a line marked "privacy: allow", so this file passes the check itself.


class Paths(unittest.TestCase):
    def test_personal_folder_paths_are_found(self):
        for text in [
            r"see C:\Users\alice\Documents\notes.txt",  # privacy: allow
            "see C:/Users/alice/project",  # privacy: allow
            "/home/alice/.cargo/bin",  # privacy: allow
            "/Users/alice/Library",  # privacy: allow
        ]:
            line = text.replace("# privacy: allow", "")
            self.assertTrue(scan_text("f", line), text)

    def test_generic_paths_are_fine(self):
        for line in [r"C:\Users\Public\Desktop", "/home/runner/work", r"C:\Users\<name>\.claude",
                     "~/.cargo/bin", "$HOME/.cargo/bin", "/usr/bin/env", "src/Users/list.rs"]:
            self.assertEqual(scan_text("f", line), [], line)


class Emails(unittest.TestCase):
    def test_real_addresses_are_found(self):
        self.assertTrue(scan_text("f", "Contact " + "alice" + "@" + "mail.test.io"))

    def test_noreply_and_example_addresses_are_fine(self):
        for line in ["350777876+bot@users.noreply.github.com", "Co-Authored-By: Claude <noreply@anthropic.com>",
                     "someone@example.com", "git@github.com:owner/repo.git", "icons/128x128@2x.png"]:
            self.assertEqual(scan_text("f", line), [], line)


class Words(unittest.TestCase):
    def test_private_words_match_whole_words_case_insensitively(self):
        self.assertTrue(scan_text("f", "Built on Zebedee's laptop", ["zebedee"]))
        self.assertEqual(scan_text("f", "zebedees", ["zebedee"]), [])

    def test_words_file_ignores_comments_and_blanks(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / ".privacy-words"
            p.write_text("# my name\nZebedee\n\n  laptop-7  \n", encoding="utf-8")
            self.assertEqual(private_words(p), ["Zebedee", "laptop-7"])
            self.assertEqual(private_words(Path(d) / "missing"), [])

    def test_allow_marker_skips_a_line(self):
        self.assertEqual(scan_text("f", "Zebedee  # privacy: allow", ["zebedee"]), [])

    def test_findings_name_file_line_and_kind(self):
        finding = scan_text("docs/x.md", "ok\nby Zebedee", ["zebedee"])[0]
        self.assertEqual(finding, "docs/x.md:2: private word from .privacy-words: zebedee")


if __name__ == "__main__":
    unittest.main()
