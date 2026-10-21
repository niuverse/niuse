+++
id = "M2-030"
title = "Status and loops JSON"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-005"]
docs = ["architecture"]
+++

## Scope

- `niuse status --json` emits nested `memory`, `loops`, and `prompt` sections plus home, thread, identity, and user flags.
- `niuse loops list --json` emits `id`, `text`, `due`, `done`, and `overdue`.
- Human output is unchanged when the flag is absent.
- Both commands are covered by end-to-end CLI tests.

## Acceptance

- [x] Status JSON parses and reports zero-state counts.
- [x] Loops JSON round-trips a due date.
- [x] Overdue is computed at read time.
- [x] Existing text output is untouched.

## Notes

This closes the JSON surface: memory, journal, threads, loops, status, doctor, and stats all have a machine-readable view.
