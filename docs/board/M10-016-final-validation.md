+++
id = "M10-016"
title = "Final validation"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P0"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-015"]
docs = ["roadmap"]
+++

## Problem

The last round needs a clean-tree validation at the exact candidate commits, not
an earlier run.

## Scope

- Run `release-preflight.sh` from clean trees.
- Record heads, pin, gates, and dogfood.
- Confirm only human/calendar gates remain.
- No bump, publish, or dogfood edit.

## Acceptance

- [x] Preflight exits 0.
- [x] Heads and pin recorded.
- [x] All gates pass.
- [x] Dogfood remains 2/10 and is stated.
- [x] Verdict limits completion to technical scope.

## Notes

Completion of the goal does not imply publication.
