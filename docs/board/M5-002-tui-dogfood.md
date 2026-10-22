+++
id = "M5-002"
title = "Ten TUI dogfood days"
status = "doing"
milestone = "M5"
module = "cli"
size = "S"
priority = "P0"
updated = "2026-10-23"
depends_on = ["M5-001"]
docs = ["dogfood", "adr-0004"]
+++

## Problem

The TUI path passes synthetic PTY tests, but ADR-0004 requires ten real working
days before the renderer can be removed or the TUI can become the default.
Synthetic turns cannot reveal candidate fatigue, approval confusion, or
terminal-specific rendering failures.

## Scope

- Use `niuse tui` for at least one real task on ten working days.
- Record each day with `./scripts/dogfood.sh "note"`, including the task, the
  outcome, and one concrete memory win or disagreement.
- Note every TUI-specific failure; do not round a partial day up.
- Do not delete `render.rs` until the gate is complete.

## Acceptance

- [ ] Ten distinct working days are recorded, with at least one real task each.
- [ ] At least three entries cite a past judgement correctly.
- [ ] At least one entry records a disagreement with a reason.
- [ ] The TUI path has no unresolved approval or data-loss failure.
- [ ] The gate result is recorded in `docs/dogfood.md` and the ADR-0004 decision
      event.

## Notes

2026-10-22: day 1/10. One real TUI turn replied `391` through the provider
(13k input, 2 output). This is a recorded TUI day, not a synthetic replay.

2026-10-23: day 2/10. A real TUI task read `DAY_TWO_C.md` and answered with its
heading (27k input, 47 output, 13.4k cached). The turn initially recorded 0
metrics; M5-010 added the missing `on_turn_end`. Eight distinct working days
remain; the same calendar day cannot count twice.

If the gate fails, keep the REPL default and open a focused card for the failure
instead of extending the gate's definition.
