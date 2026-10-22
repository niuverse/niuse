#!/usr/bin/env python3
"""Check milestone and card states for contradictions the dashboard misses."""
import argparse
import json
import sys
import tomllib
from pathlib import Path

CARD_STATUSES = {"backlog", "ready", "doing", "review", "blocked", "done"}
MILESTONE_STATUSES = {"planned", "active", "done"}


def frontmatter(path):
    text = path.read_text(errors="replace")
    if not text.startswith("+++"):
        raise ValueError("missing +++ frontmatter")
    block = text.split("+++", 2)[1]
    return tomllib.loads(block)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    args = parser.parse_args()
    root = Path(args.root)
    project = json.loads((root / "docs/dashboard/project.json").read_text())
    milestones = {m["id"]: m["status"] for m in project["milestones"]}
    errors = []

    active = [mid for mid, status in milestones.items() if status == "active"]
    if len(active) > 1:
        errors.append(f"multiple active milestones: {', '.join(active)}")
    for mid, status in milestones.items():
        if status not in MILESTONE_STATUSES:
            errors.append(f"{mid}: bad milestone status {status}")

    cards = []
    for path in sorted((root / "docs/board").glob("*.md")):
        card = frontmatter(path)
        cid = card.get("id", path.stem)
        status = card.get("status")
        mid = card.get("milestone")
        cards.append(cid)
        if status not in CARD_STATUSES:
            errors.append(f"{cid}: bad card status {status!r}")
        if mid not in milestones:
            errors.append(f"{cid}: unknown milestone {mid!r}")
            continue
        if status == "done" and not card.get("completed"):
            errors.append(f"{cid}: done without completed")
        if status == "blocked" and not card.get("blocked_reason"):
            errors.append(f"{cid}: blocked without blocked_reason")
        if status == "doing" and milestones[mid] == "planned":
            errors.append(f"{cid}: doing in planned milestone {mid}")
        if milestones[mid] == "done" and status != "done":
            errors.append(f"{cid}: {status} card in done milestone {mid}")

    if len(cards) != len(set(cards)):
        errors.append("duplicate card ids")
    for error in errors:
        print(f"board error: {error}", file=sys.stderr)
    print(
        f"board check: {len(cards)} cards, {len(milestones)} milestones, "
        f"{'ok' if not errors else f'{len(errors)} error(s)'}"
    )
    sys.exit(1 if errors else 0)


if __name__ == "__main__":
    main()
