+++
id = "M2-039"
title = "Session token totals with /cost"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-032"]
docs = ["architecture"]
+++

## Scope

- `/cost` sums the `turn_end` usage already recorded in the current thread log.
- It reports turns, input, output, and cached tokens; compact counts like `3.6k`.
- Resumed threads include their earlier turns, because the log is the source.
- No in-memory counter is added; the aggregation is a pure function over events.

## Acceptance

- [x] The dogfood thread reported `2 turns · 3.6k in · 149 out`.
- [x] Two synthetic events aggregate correctly in a unit test.
- [x] `/help` and completion list the command.
- [x] No new state or dependency.

## Notes

Per-turn usage is for pacing; `/cost` is for "what has this thread actually spent". Both read the same recorded events.
