+++
id = "M2-047"
title = "Grouped help"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-031"]
docs = ["architecture"]
+++

## Scope

- `/help` is grouped by purpose: conversation, memory, promises/journal, session, model/cost, identity/settings, debugging.
- The duplicate `/export` entry is gone and columns line up.
- Help lives in one raw string; a test asserts every command appears.
- No command behavior changes.

## Acceptance

- [x] Every documented command is present in the help text.
- [x] The continuation hint and review keys are correct.
- [x] The test fails if a future command is omitted from the list check.
- [x] Output remains plain text.

## Notes

The command count crossed twenty. A flat list stops being scannable before it stops being useful.
