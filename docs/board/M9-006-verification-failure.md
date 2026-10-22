+++
id = "M9-006"
title = "Verification failure recovery"
status = "done"
milestone = "M9"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M9-005"]
docs = ["evals", "roadmap"]
+++

## Problem

A step with a failed verification was rendered as `no recorded verification`,
which looks like nothing ran. The step stayed open, but the UI hid the failure
and invited the model to retry blindly.

## Scope

- `Evidence::step_state`: verified / verification failed / no recorded
  verification.
- `evidence` and TUI `/steps` use the same state.
- A failed check keeps the step in the next prompt and refuses `plan done`.
- A later pass closes it; both the failed and passing checks stay in history.

## Acceptance

- [x] A failed-only step says `verification failed`, not "none".
- [x] The step remains in the prompt after the failure.
- [x] `plan done` is refused until a pass exists.
- [x] A later passing check closes the step and removes the plan section.
- [x] Evidence JSON keeps both checks in order.
- [x] TUI `/steps` shows the failed state.

## Notes

Failure is history, not an alarm: the passing check makes the step verified
while the failed attempt remains visible. No automatic retry is introduced.
