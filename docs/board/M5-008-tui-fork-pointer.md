+++
id = "M5-008"
title = "TUI /fork pointer PTY"
status = "done"
milestone = "M5"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M5-007"]
docs = ["roadmap", "evals"]
+++

## Problem

`/fork` was wired to `on_session_start`, but the unit test only proved the
notification helper. The real risk is a product command writing to the source
thread after a fork.

## Scope

- PTY: run one turn, `/fork`, then `/verify p-1 true`.
- The verification must land in the fork destination, not the source thread.
- Plan p-1 is created in the temp home before the TUI starts.

## Acceptance

- [x] `/fork` prints `fork ·` and opens the destination.
- [x] `/verify p-1` writes `verification` with `step: p-1` to the fork file.
- [x] The source `thread.jsonl` has no verification event.
- [x] The fork contains `FORK_SOURCE_ANSWER`.
- [x] Runs under `cargo test -p niuse --features tui`.

## Notes

ADR-0007 still applies: the fork is the live session for this run, not the next
launch's `thread.jsonl`.
