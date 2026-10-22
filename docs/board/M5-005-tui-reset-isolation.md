+++
id = "M5-005"
title = "TUI /reset session isolation"
status = "done"
milestone = "M5"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M5-004"]
docs = ["roadmap", "evals"]
+++

## Problem

`/new` was proven to archive the old thread and reset `thread.jsonl`, but
`/reset` shares the same `new_session_path` seam and had no PTY coverage. A
regression there would leave plan/goal cleared while product commands still
pointed at the old log.

## Scope

- Parameterize the PTY test for `/new` and `/reset`.
- Both commands must archive the old thread, leave only the new answer in
  `thread.jsonl`, and keep the old answer only in the archive.
- The hook pointer must be updated by `on_session_start` for both.

## Acceptance

- [x] `/new` PTY isolation passes.
- [x] `/reset` PTY isolation passes.
- [x] Both archive the old log under `threads/`.
- [x] `thread.jsonl` contains only the second answer in both cases.
- [x] One script drives both modes; no copy-pasted harness.

## Notes

`/reset` additionally clears plan/goal/history, but the session-path behavior is
the risk this test covers. The parser and new-session seam remain unchanged.
