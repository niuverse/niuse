+++
id = "M7-005"
title = "Verification-gated plan ledger"
status = "done"
milestone = "M7"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M7-004"]
docs = ["roadmap", "evals"]
+++

## Problem

The plan existed only in the transcript. A step could be called done with no
durable evidence, and `verification` events were not tied to the step they
proved.

## Scope

- `plan.md`: human-owned steps with `p-N` labels and a done bit.
- `niuse plan list|add|done`, capped open steps, secret-scanned add.
- `niuse verify --step p-N` tags the verification event.
- `plan done p-N` refuses unless the current thread has a passing verification
  for that step.
- `niuse evidence` renders plan steps with `verified` / `no recorded
  verification`.

## Acceptance

- [x] Steps get stable `p-N` labels and survive reload.
- [x] `plan done` is refused without a passing step-tagged verification.
- [x] A passing verification closes the step.
- [x] Evidence JSON carries both plan state and the verification's step.
- [x] The CLI lifecycle test covers add → refused → verify → done.
- [x] Existing verification events without a step still parse.

## Notes

2026-10-22 real run: with `p-1` injected (M7-006), the model read the file,
refused to mark the step done because it had no verification evidence, and
asked which direction to take. The human ran `verify --step p-1 -- grep ...`,
the check passed, and `plan done 1` closed the step. This is the gate working
against a real provider, not a scripted turn.
