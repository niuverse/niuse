+++
id = "M7-003"
title = "Thread evidence report"
status = "done"
milestone = "M7"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M7-002"]
docs = ["roadmap", "evals"]
+++

## Problem

After a long execute phase, the final answer is a claim. The thread already
contains what actually ran: which tools were called, which files were touched,
which checks ran, what failed, and which approvals were denied. That evidence
was visible only by scrolling the transcript.

## Scope

- Read-only `Evidence` derived from `assistant_message.tool_calls` and
  `tool_result` events.
- List write/edit paths, test-like bash checks with pass/fail, total failures,
  and denied approvals.
- `niuse evidence [--thread PATH] [--json]`, `/evidence` in the REPL and TUI.
- No new store and no new event type.

## Acceptance

- [x] A thread with a write and `cargo test` reports one file and one passing
      check.
- [x] A failed tool and a denied approval are counted separately.
- [x] Empty threads say there is no evidence instead of implying success.
- [x] JSON is stable for scripts.
- [x] The report is available in the REPL and TUI.

## Notes

This separates "ran and succeeded" from "the answer said so". It does not prove
the task is correct; that still needs a real verification command chosen for
the task. Test-like commands are a heuristic over bash text.
