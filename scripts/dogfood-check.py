#!/usr/bin/env python3
"""Validate the dogfood log; report progress without fabricating days."""
import argparse
import datetime
import json
import re
import sys
from pathlib import Path

HEADER = "| date | turns | proposed | accepted | note |"
DATE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
MEMORY = re.compile(r"\bm-[0-9a-f]{4}\b", re.IGNORECASE)


def fail(message):
    print(f"dogfood check: FAIL · {message}", file=sys.stderr)
    sys.exit(1)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--file", default="docs/dogfood.md")
    parser.add_argument("--today", default=datetime.date.today().isoformat())
    parser.add_argument("--complete", action="store_true")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    today = datetime.date.fromisoformat(args.today)
    path = Path(args.file)
    if not path.is_file():
        fail(f"missing {path}")

    lines = path.read_text().splitlines()
    try:
        start = next(index for index, line in enumerate(lines) if line.strip() == HEADER)
    except StopIteration:
        fail("missing dogfood table header")
    rows = []
    for line in lines[start + 2 :]:
        if not line.lstrip().startswith("|"):
            break
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) != 5:
            fail(f"row has {len(cells)} cells: {line}")
        date_text, turns_text, proposed_text, accepted_text, note = cells
        if not DATE.match(date_text):
            fail(f"bad date: {date_text}")
        try:
            date = datetime.date.fromisoformat(date_text)
            turns = int(turns_text)
            proposed = int(proposed_text)
            accepted = int(accepted_text)
        except ValueError as error:
            fail(f"bad row {date_text}: {error}")
        if date > today:
            fail(f"future day: {date_text}")
        if turns < 0 or proposed < 0 or accepted < 0:
            fail(f"negative count on {date_text}")
        if accepted > proposed:
            fail(f"accepted > proposed on {date_text}")
        rows.append(
            {
                "date": date,
                "turns": turns,
                "proposed": proposed,
                "accepted": accepted,
                "note": note,
            }
        )

    dates = [row["date"] for row in rows]
    if len(set(dates)) != len(dates):
        fail("duplicate date")
    rows.sort(key=lambda row: row["date"])
    proposed = sum(row["proposed"] for row in rows)
    accepted = sum(row["accepted"] for row in rows)
    acceptance = (100 * accepted // proposed) if proposed else None
    cited = sum(1 for row in rows if MEMORY.search(row["note"]))
    disagreements = sum(
        1 for row in rows if "disagree" in row["note"].lower()
    )

    problems = []
    if len(rows) < 10:
        problems.append(f"only {len(rows)}/10 days")
    if acceptance is None or acceptance < 60:
        problems.append(
            f"acceptance {acceptance if acceptance is not None else 'n/a'} < 60"
        )
    if cited < 3:
        problems.append(f"only {cited} cited memory ids (< 3)")
    if disagreements < 1:
        problems.append("no recorded disagreement")

    payload = {
        "days": len(rows),
        "required_days": 10,
        "proposed": proposed,
        "accepted": accepted,
        "acceptance_percent": acceptance,
        "cited_memory_ids": cited,
        "disagreements": disagreements,
        "complete": not problems,
        "problems": problems,
    }
    if args.json:
        print(json.dumps(payload, indent=2))
    else:
        print(f"dogfood days: {len(rows)}/10")
        print(
            "acceptance:   "
            + (f"{acceptance}% ({accepted}/{proposed})" if acceptance is not None else "n/a (no proposals)")
        )
        print(f"notes:        {cited} cited m-ids · {disagreements} disagreement(s)")
        if problems:
            for problem in problems:
                print(f"gap:          {problem}")
            print("status:       in progress")
        else:
            print("status:       complete (human review still required)")
    if args.complete and problems:
        sys.exit(1)


if __name__ == "__main__":
    main()
