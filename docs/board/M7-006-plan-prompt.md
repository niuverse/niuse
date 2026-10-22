+++
id = "M7-006"
title = "Inject the open plan into the prompt"
status = "done"
milestone = "M7"
module = "prompt"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M7-005"]
docs = ["roadmap", "evals"]
+++

## Problem

`plan.md` was a human ledger. The agent never saw the steps, so it could not
work toward them or warn that a step had no verification.

## Scope

- `compose_with_plan`: goals, then open plan steps, then memory.
- Only open steps, at most six, within a 300-token budget.
- State the completion rule: verify with `niuse verify --step p-N` before
  claiming a step is done.
- Pass the plan through the REPL, TUI, `/prompt`, and eval composition.

## Acceptance

- [x] Open steps appear under `当前计划` with their `p-N` labels.
- [x] Done steps are omitted.
- [x] An all-done plan adds no section.
- [x] The section states the verification command.
- [x] The plan sits between goals and memory.
- [x] The offline eval covers the injection.

## Notes

The plan is still human-owned; the model cannot add or close steps. That
mutation stays behind `niuse plan`, so the verification gate cannot be
self-approved.
