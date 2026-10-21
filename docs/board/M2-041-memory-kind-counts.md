+++
id = "M2-041"
title = "Kind counts in the memory header"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-003"]
docs = ["architecture"]
+++

## Scope

- `niuse mem list` shows a per-kind breakdown in the header, e.g. `6 条记忆（preference 2 · lesson 4）`.
- An empty store prints no parenthetical.
- JSON output is unchanged; the summary is text-only.
- The counting helper has a unit test.

## Acceptance

- [x] Preference and lesson counts come from the loaded entries.
- [x] Empty stores stay terse.
- [x] `--kind` filtering still leaves the global header intact.
- [x] No storage change.

## Notes

The first question about a memory store is usually "is this mostly preferences or lessons". Answering it in the header avoids a second command.
