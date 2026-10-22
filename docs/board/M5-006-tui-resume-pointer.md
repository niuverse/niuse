+++
id = "M5-006"
title = "TUI /resume pointer PTY"
status = "done"
milestone = "M5"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M5-005"]
docs = ["roadmap", "evals"]
+++

## Problem

`/resume` was wired to `on_session_start`, but only a unit test checked the
notification. The real risk is a product command writing to the wrong thread
after the switch.

## Scope

- PTY: run one turn, `/new` to archive it, run a second turn, `/resume` the
  archived file, then `/verify p-1 true`.
- The verification must land in the resumed file, not the active thread.
- Plan p-1 is created in the temp home before the TUI starts.

## Acceptance

- [x] `/resume` prints `resumed` and switches the session.
- [x] `/verify p-1` writes `verification` with `step: p-1` to the resumed file.
- [x] The active `thread.jsonl` has no verification event.
- [x] The archived FIRST_ANSWER survives.
- [x] Runs under `cargo test -p niuse --features tui`.

## Notes

Cross-restart active-thread semantics are still separate: exiting after
`/resume` leaves `thread.jsonl` as the active file. This test proves the
in-session pointer, which is what `/evidence`, `/verify`, and plan commands use.
