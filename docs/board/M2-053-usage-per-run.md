+++
id = "M2-053"
title = "Usage is per run, not per step"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-032", "M2-039"]
docs = ["architecture"]
+++

## Scope

- Found by a self-dogfood write task: one prompt produced four usage lines and `/cost` reported four turns.
- `AgentEvent::TurnEnd` fires per model step, not per user turn.
- The renderer now accumulates usage and prints one line per run: `2 steps · 3.9k in · …`.
- `/cost` counts `user_message` events as turns and still sums `turn_end` usage for tokens.

## Acceptance

- [x] A multi-step run prints exactly one usage line.
- [x] `/cost` counts user messages, not steps.
- [x] The aggregation test includes two user messages and two steps.
- [x] Token totals are unchanged in meaning.

## Notes

"Turn" meant two different things in two places. The transcript now shows steps explicitly; `/cost` answers "how much did this conversation spend".
