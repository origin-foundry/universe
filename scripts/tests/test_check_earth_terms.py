import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_earth_terms import scan_text, words  # noqa: E402

# This file sits outside the folders the check scans, so its examples don't trip it.


class Words(unittest.TestCase):
    def test_identifiers_are_split_into_words(self):
        self.assertEqual(list(words("year_def")), ["year", "def"])
        self.assertEqual(list(words("HUNDRED_THOUSAND_YEARS")), ["hundred", "thousand", "years"])
        self.assertEqual(list(words("LeapRuleDef")), ["leap", "rule", "def"])
        self.assertEqual(list(words("hoursFromOne")), ["hours", "from", "one"])


class Findings(unittest.TestCase):
    def test_earth_words_are_found_in_code_comments_and_strings(self):
        for line in ['let year = 4;', '// the month after', 'name: "hour".to_string(),',
                     'fn convert_century_numbers()', 'pub struct LeapRuleDef', 'const DAYS: i64 = 7;',
                     'const HUNDRED_THOUSAND_YEARS: i64 = 1;', '"every Monday"']:
            self.assertTrue(scan_text("crates/x.rs", line), line)

    def test_the_finding_names_the_file_line_and_words(self):
        self.assertEqual(scan_text("crates/x.rs", "ok\nlet years = months;"),
                         ["crates/x.rs:2: Earth word: months, years"])

    def test_generic_words_are_fine(self):
        for line in ["let era_number = 4;", "// one second, the SI unit", "members, fundamental, subdivision",
                     "today and yesterday", "orbit % 4 == 0", "hourglass", "dayside", "Mayday", "seconds"]:
            self.assertEqual(scan_text("crates/x.rs", line), [], line)

    def test_lines_about_earth_or_gregorian_are_allowed(self):
        for line in ["// The era's unit (on Earth, the year).", "fn gregorian_shaped() -> CalendarDef { // year",
                     "const GREGORIAN: &str = \"year % 4 == 0\";"]:
            self.assertEqual(scan_text("crates/x.rs", line), [], line)

    def test_files_named_gregorian_are_allowed(self):
        self.assertEqual(scan_text("crates/uni-calendar/tests/gregorian.rs", "let year = 1;"), [])
        self.assertEqual(scan_text("crates/uni-calendar/tests/data/Gregorian_days.py", "month"), [])

    def test_off_and_on_markers_allow_a_block(self):
        text = "// earth-terms: off\nlet year = 1;\nlet month = 2;\n// earth-terms: on\nlet day = 3;"
        self.assertEqual(scan_text("crates/x.rs", text), ["crates/x.rs:5: Earth word: day"])

    def test_a_line_naming_both_markers_only_mentions_them(self):
        text = "// use `earth-terms: off` and `earth-terms: on`\nlet year = 1;"
        self.assertEqual(scan_text("crates/x.rs", text), ["crates/x.rs:2: Earth word: year"])

    def test_an_unclosed_block_is_an_error(self):
        findings = scan_text("crates/x.rs", "// earth-terms: off\nlet year = 1;")
        self.assertEqual(len(findings), 1)
        self.assertIn("never followed", findings[0])


if __name__ == "__main__":
    unittest.main()
