+++
id = "M9-005"
title = "Multi-turn plan execution eval"
status = "done"
milestone = "M9"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M9-004"]
docs = ["evals", "roadmap"]
+++

## Problem

The combined workflow proved a single turn. A plan that spans turns can fail
differently: a completed step lingers in the prompt, or the next step runs
without the previous one being verified.

## Scope

- Two real plan steps and two MCP packs.
- Turn one calls notes, the test verifies and closes p-1.
- Recompose: p-1 must disappear, p-2 must remain.
- Turn two calls calendar, the test verifies and closes p-2.
- Recompose: no `当前计划` section remains.
- Evidence reports two done steps and two step-tagged verifications.

## Acceptance

- [x] Both steps are present before turn one.
- [x] After p-1 closes, only p-2 is in the next prompt.
- [x] After p-2 closes, no plan section is injected.
- [x] Both tool results are in the thread.
- [x] Evidence shows both steps done and both verifications tagged.
- [x] The eval runner includes the case.

## Notes

The human closes steps; the model cannot self-approve. This case checks the
prompt transition between turns, which a single-turn test cannot see.
