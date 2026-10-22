#!/usr/bin/env python3
"""Fail when a relative Markdown link in README/AGENTS/docs is missing."""
import argparse
import re
import sys
from pathlib import Path

LINK = re.compile(r"\[[^\]]*\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")
SKIP = ("http://", "https://", "#", "mailto:")


def markdown_files(root):
    files = [root / "README.md", root / "AGENTS.md"]
    files.extend(sorted((root / "docs").rglob("*.md")))
    return [path for path in files if path.is_file()]


def links(text):
    fenced = False
    for line in text.splitlines():
        if line.strip().startswith("```"):
            fenced = not fenced
            continue
        if fenced:
            continue
        yield from LINK.findall(line)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    args = parser.parse_args()
    root = Path(args.root).resolve()
    missing = []
    for path in markdown_files(root):
        for target in links(path.read_text(errors="replace")):
            if target.startswith(SKIP):
                continue
            target = target.split("#", 1)[0]
            if not target:
                continue
            resolved = (path.parent / target).resolve()
            if resolved != root and root not in resolved.parents:
                continue
            if not resolved.exists():
                missing.append((path.relative_to(root), target))
    for path, target in missing:
        print(f"missing link: {path} -> {target}", file=sys.stderr)
    print(f"link check: {'ok' if not missing else f'{len(missing)} missing'}")
    sys.exit(1 if missing else 0)


if __name__ == "__main__":
    main()
