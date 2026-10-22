+++
id = "M7-007"
title = "Delegation evidence"
status = "done"
milestone = "M7"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M7-003"]
docs = ["roadmap", "evals"]
+++

## Problem

niucode's `subagent` tool already delegates read-only work, but the parent
thread recorded spawn/result events that neither the human nor `niuse evidence`
surfaced. Delegated work was invisible unless the transcript was scrolled.

## Scope

- Read `subagent_spawn` / `subagent_result` from the thread.
- Report profile, task, session, turns, ok, and summary.
- A spawn without a result reports `?`, never success.
- `niuse evidence --json` includes the delegation array.
- Pin `subagent` availability in the niucode contract test.

## Acceptance

- [x] Completed explore runs show ✓ and their summary.
- [x] An unfinished spawn shows `?` and no success claim.
- [x] The task, profile, session, and turn count are visible.
- [x] JSON carries the same data.
- [x] The contract test fails if the `subagent` tool disappears.

## Notes

This is read-only delegation through niucode's tool; niuse does not add a second
orchestrator. Worker writes remain a niucode concern with its own write lock.
