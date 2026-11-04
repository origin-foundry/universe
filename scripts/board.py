#!/usr/bin/env python3
"""Move and add cards on the Universe project board, as the bot.

    python scripts/board.py show 11
    python scripts/board.py move 11 "Design review"
    python scripts/board.py add 24 --status Backlog --phase "0. Foundations" --area storage --size S

Works for issues and pull requests. Enforces the feature-workflow rule that Claude never moves a
card out of a review column (knowledge/process/feature-workflow.md): those moves are the owner's.

It calls the GitHub API through `gh api graphql`, because `gh project` commands don't work for
projects owned by a personal account.
"""
import argparse
import json
import subprocess
import sys

OWNER, REPO, PROJECT = "origin-foundry", "universe", 3
REVIEW_COLUMNS = {"Spec review", "Design review", "PR review"}
FIELDS = {"status": "Status", "phase": "Phase", "area": "Area", "size": "Size"}

PROJECT_QUERY = """query{user(login:"%s"){projectV2(number:%d){id
  fields(first:30){nodes{... on ProjectV2SingleSelectField{id name options{id name}}}}}}}""" % (OWNER, PROJECT)
ITEM_QUERY = """query($n:Int!){repository(owner:"%s",name:"%s"){issueOrPullRequest(number:$n){
  ... on Issue{id title projectItems(first:10){nodes{id project{number}
    fieldValueByName(name:"Status"){... on ProjectV2ItemFieldSingleSelectValue{name}}}}}
  ... on PullRequest{id title projectItems(first:10){nodes{id project{number}
    fieldValueByName(name:"Status"){... on ProjectV2ItemFieldSingleSelectValue{name}}}}}}}}""" % (OWNER, REPO)
ADD_MUTATION = "mutation($p:ID!,$c:ID!){addProjectV2ItemById(input:{projectId:$p,contentId:$c}){item{id}}}"
SET_MUTATION = """mutation($p:ID!,$i:ID!,$f:ID!,$o:String!){updateProjectV2ItemFieldValue(
  input:{projectId:$p,itemId:$i,fieldId:$f,value:{singleSelectOptionId:$o}}){projectV2Item{id}}}"""


class BoardError(Exception):
    pass


def gh_graphql(query, **variables):
    args = ["gh", "api", "graphql", "-f", f"query={query}"]
    for key, value in variables.items():
        args += ["-F" if isinstance(value, int) else "-f", f"{key}={value}"]
    result = subprocess.run(args, capture_output=True, text=True, encoding="utf-8")
    if result.returncode != 0:
        raise BoardError(result.stderr.strip() or "gh api graphql failed")
    return json.loads(result.stdout)["data"]


def option_ids(project, field, option):
    """(field id, option id) for a single-select field's option, by name."""
    for node in project["fields"]["nodes"]:
        if node and node.get("name") == field:
            for opt in node["options"]:
                if opt["name"] == option:
                    return node["id"], opt["id"]
            names = ", ".join(o["name"] for o in node["options"])
            raise BoardError(f"{field} has no option {option!r} (options: {names})")
    raise BoardError(f"the board has no field {field!r}")


def check_move(current, target):
    """Refuse moves the workflow reserves for the owner."""
    if current in REVIEW_COLUMNS and target != current:
        raise BoardError(
            f"the card is in {current!r}; only the owner moves cards out of review columns "
            "(knowledge/process/feature-workflow.md)"
        )


class Board:
    def __init__(self, graphql=gh_graphql):
        self.graphql = graphql
        self.project = graphql(PROJECT_QUERY)["user"]["projectV2"]

    def card(self, number):
        """(content id, title, item id or None, current status or None)."""
        content = self.graphql(ITEM_QUERY, n=number)["repository"]["issueOrPullRequest"]
        if content is None:
            raise BoardError(f"#{number} isn't an issue or pull request in {OWNER}/{REPO}")
        for item in content["projectItems"]["nodes"]:
            if item["project"]["number"] == PROJECT:
                status = (item.get("fieldValueByName") or {}).get("name")
                return content["id"], content["title"], item["id"], status
        return content["id"], content["title"], None, None

    def set_field(self, item, field, option):
        field_id, option_id = option_ids(self.project, field, option)
        self.graphql(SET_MUTATION, p=self.project["id"], i=item, f=field_id, o=option_id)

    def move(self, number, status):
        _, title, item, current = self.card(number)
        if item is None:
            raise BoardError(f"#{number} isn't on the board; use 'add' first")
        check_move(current, status)
        self.set_field(item, "Status", status)
        return f"#{number} {title}: {current or '(none)'} -> {status}"

    def add(self, number, **fields):
        content, title, item, current = self.card(number)
        if item is not None and fields.get("status"):
            check_move(current, fields["status"])
        if item is None:
            item = self.graphql(ADD_MUTATION, p=self.project["id"], c=content)["addProjectV2ItemById"]["item"]["id"]
        for key, value in fields.items():
            if value:
                self.set_field(item, FIELDS[key], value)
        shown = ", ".join(f"{FIELDS[k]}={v}" for k, v in fields.items() if v)
        return f"#{number} {title}: on the board ({shown or 'no fields set'})"

    def show(self, number):
        _, title, item, status = self.card(number)
        return f"#{number} {title}: " + (status or "(no status)" if item else "not on the board")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("show").add_argument("number", type=int)
    move = sub.add_parser("move")
    move.add_argument("number", type=int)
    move.add_argument("status")
    add = sub.add_parser("add")
    add.add_argument("number", type=int)
    for key in FIELDS:
        add.add_argument(f"--{key}")
    args = parser.parse_args(argv)
    try:
        board = Board()
        if args.command == "show":
            print(board.show(args.number))
        elif args.command == "move":
            print(board.move(args.number, args.status))
        else:
            print(board.add(args.number, **{k: getattr(args, k) for k in FIELDS}))
    except BoardError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
