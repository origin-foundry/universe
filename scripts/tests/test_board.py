import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from board import ADD_MUTATION, SET_MUTATION, Board, BoardError  # noqa: E402

PROJECT = {
    "id": "P1",
    "fields": {"nodes": [
        {"id": "F-status", "name": "Status", "options": [
            {"id": f"S-{n}", "name": n} for n in
            ["Backlog", "Specifying", "Spec review", "Planning", "Design review", "Building", "PR review", "Done"]]},
        {"id": "F-area", "name": "Area", "options": [{"id": "A-storage", "name": "storage"}]},
        {},  # non-select fields come back empty
    ]},
}


class FakeGitHub:
    def __init__(self, status=None, on_board=True, exists=True):
        self.status, self.on_board, self.exists = status, on_board, exists
        self.mutations = []

    def __call__(self, query, **variables):
        if query.startswith("query{user"):
            return {"user": {"projectV2": PROJECT}}
        if "issueOrPullRequest" in query:
            if not self.exists:
                return {"repository": {"issueOrPullRequest": None}}
            items = [{"id": "I1", "project": {"number": 3},
                      "fieldValueByName": {"name": self.status} if self.status else None}] if self.on_board else []
            return {"repository": {"issueOrPullRequest": {
                "id": "C1", "title": "Project file", "projectItems": {"nodes": items}}}}
        self.mutations.append((query, variables))
        if query == ADD_MUTATION:
            return {"addProjectV2ItemById": {"item": {"id": "I-new"}}}
        return {}


class Moves(unittest.TestCase):
    def test_moves_into_a_review_column(self):
        gh = FakeGitHub(status="Planning")
        self.assertIn("Planning -> Design review", Board(gh).move(11, "Design review"))
        self.assertEqual(gh.mutations, [(SET_MUTATION, {"p": "P1", "i": "I1", "f": "F-status", "o": "S-Design review"})])

    def test_refuses_to_move_out_of_a_review_column(self):
        for column in ("Spec review", "Design review", "PR review"):
            gh = FakeGitHub(status=column)
            with self.assertRaisesRegex(BoardError, "only the owner moves cards out of review columns"):
                Board(gh).move(11, "Building")
            self.assertEqual(gh.mutations, [], column)

    def test_unknown_status_names_the_options(self):
        with self.assertRaisesRegex(BoardError, "no option 'Bulding'.*Building"):
            Board(FakeGitHub(status="Planning")).move(11, "Bulding")

    def test_card_not_on_board_or_missing(self):
        with self.assertRaisesRegex(BoardError, "isn't on the board"):
            Board(FakeGitHub(on_board=False)).move(11, "Planning")
        with self.assertRaisesRegex(BoardError, "isn't an issue or pull request"):
            Board(FakeGitHub(exists=False)).move(999, "Planning")


class Adds(unittest.TestCase):
    def test_adds_and_sets_fields(self):
        gh = FakeGitHub(on_board=False)
        Board(gh).add(24, status="Backlog", area="storage")
        self.assertEqual(gh.mutations[0], (ADD_MUTATION, {"p": "P1", "c": "C1"}))
        self.assertEqual([v["o"] for q, v in gh.mutations[1:]], ["S-Backlog", "A-storage"])
        self.assertEqual([v["i"] for q, v in gh.mutations[1:]], ["I-new", "I-new"])

    def test_add_on_a_card_in_review_still_refuses_to_move_it(self):
        gh = FakeGitHub(status="PR review")
        with self.assertRaises(BoardError):
            Board(gh).add(11, status="Done")
        self.assertEqual(gh.mutations, [])


class Show(unittest.TestCase):
    def test_show(self):
        self.assertIn("Design review", Board(FakeGitHub(status="Design review")).show(11))
        self.assertIn("not on the board", Board(FakeGitHub(on_board=False)).show(11))


if __name__ == "__main__":
    unittest.main()
