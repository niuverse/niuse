+++
id = "M7-009"
title = "TUI plan/verify PTY regression"
status = "done"
milestone = "M7"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M7-008"]
docs = ["roadmap", "evals"]
+++

## Problem

`/steps` and step-tagged `/verify` were covered only at the hook level. A real
terminal also has completion, redraw, and command routing, and `/evidence`
forgot to attach the plan store, so the PTY test exposed a gap.

## Scope

- PTY test types the full flow: add → refused done → verify → done → evidence.
- Assert the plan file, the thread verification, and the visible states.
- Fix TUI `/evidence` to attach `store.plan()` like the CLI command.

## Acceptance

- [x] `/steps add` shows `added p-1`.
- [x] `/steps` shows `no recorded verification`.
- [x] `/steps done` is refused with `no passing verification`.
- [x] `/verify p-1 true` passes and `/steps done 1` closes.
- [x] `/steps` then says `no open steps`.
- [x] `/evidence` shows the verified plan step.
- [x] `plan.md` has `[x] p-1`; the thread has the tagged verification.

## Notes

The regression found a real bug: the TUI evidence report omitted plan steps.
The test is the reason the fix exists, not a ceremonial replay.
