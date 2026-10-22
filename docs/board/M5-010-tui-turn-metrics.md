+++
id = "M5-010"
title = "TUI records turn metrics"
status = "done"
milestone = "M5"
module = "cli"
size = "S"
priority = "P0"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M5-009"]
docs = ["roadmap", "evals"]
+++

## Problem

A real TUI turn on 2026-10-23 read a file and answered, but `niuse dogfood`
recorded **0 turns**. `CompanionHooks` did not implement `on_turn_end`, so TUI
turns never reached `metrics.jsonl`; `stats`, `report`, and the dogfood gate
could not see TUI work.

## Scope

- Implement `on_turn_end`: append `tokens` when usage is non-zero and `turn ok`.
- Match the REPL's metric format.
- Unit-test through `Box<dyn TurnHooks>` so the trait-object path is covered.
- Re-run a real TUI task and wait for the metric before recording the day.

## Acceptance

- [x] TUI `on_turn_end` writes `tokens` and `turn`.
- [x] The test calls through a trait object, not only the concrete hook.
- [x] A real TUI turn writes the metric before exit.
- [x] `niuse stats --days 1` reports 1 turn.
- [x] `niuse dogfood` records 1 turn for 2026-10-23.

## Notes

The first post-fix PTY run was killed before the post-turn hook ran; the metric
appeared only after waiting for completion. The hook implementation is the
product fix; the harness now waits for it.
