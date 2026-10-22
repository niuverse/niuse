+++
id = "M7-001"
title = "Persistent human-owned goals"
status = "done"
milestone = "M7"
module = "cli"
size = "M"
priority = "P0"
updated = "2026-10-22"
completed = "2026-10-22"
docs = ["roadmap", "adr-0006", "architecture"]
+++

## Problem

`loops.md` holds the companion's promises, not the human's outcomes. niucode's
`/goal` is a session condition and disappears when a new thread starts. A
long-horizon companion needs a small, human-owned statement of what the work is
for, injected into every session.

## Scope

- `~/.niuse/goals.md` with `g-` ids, one sentence, creation date, open/done.
- `niuse goals list|add|done` and `/goals` in the REPL and TUI.
- Inject open goals after user and before memory, budget 300 tokens.
- Enforce at most seven open goals and refuse secrets at the write.
- The model reads goals but never writes them.

## Acceptance

- [x] A goal survives a new process and is injected into the prompt.
- [x] `goals add|done` and `/goals` round-trip in the REPL and TUI.
- [x] More than seven open goals is refused.
- [x] Secret-looking goals are refused and nothing is written.
- [x] `brief` and `status` report open goals.
- [x] ADR-0006 records the ownership and store decision.

## Notes

No due dates, priorities, or progress percentages. Those belong in `loops.md`
or a human project plan; this store is a compass, not a tracker.
