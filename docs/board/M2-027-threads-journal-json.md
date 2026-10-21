+++
id = "M2-027"
title = "JSON for threads and journal"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-015", "M2-008"]
docs = ["architecture"]
+++

## Scope

- `niuse threads --json` lists archives with a path relative to the niuse home, active flag, events, bytes, and age.
- `niuse journal --json` emits day objects; with `--search`, hit objects with date and text.
- The text output is unchanged when the flag is absent; `--add` ignores `--json`.
- All three persistent stores now have a machine-readable view alongside Markdown.

## Acceptance

- [x] Threads JSON uses relative paths and marks exactly one active thread.
- [x] Journal show/search JSON parse and keep dates.
- [x] Thread conversion has a unit test.
- [x] Existing text output is untouched.

## Notes

Markdown remains the only storage. JSON is a read view for scripts, not a second source of truth.
