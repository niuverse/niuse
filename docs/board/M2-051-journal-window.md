+++
id = "M2-051"
title = "Day zero means the whole journal"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-008"]
docs = ["architecture"]
+++

## Scope

- `niuse journal --days 0` lists every entry, matching `stats --days 0`.
- Positive windows keep their existing behavior.
- The window conversion is a pure helper with a unit test.
- JSON output follows the same rule.

## Acceptance

- [x] `0` maps to the full journal.
- [x] `7` still means seven days.
- [x] README documents the convention.
- [x] No storage change.

## Notes

Two commands already accepted `0` as "all time"; the journal was the odd one out. Consistency beats a special case.
