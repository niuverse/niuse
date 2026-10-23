+++
id = "stage"
title = "Stage report · 2026-10-23"
kind = "doc"
status = "living"
order = 30
summary = "The five-round checkpoint: shipped surface, real evidence, open gates, and the decisions waiting on a human."
+++

# Stage report · 2026-10-23

This is the checkpoint for the five-round close (75–79). It records what is
proven, what is still a gate, and what needs a human decision.

## Shipped and verified

- **Harness.** niucode has a read-only stdio MCP client, session hooks for
  embedders, and a new-session path seam. `/new`, `/reset`, `/resume`, and
  `/fork` all notify the embedder and are covered by PTY tests.
- **Companion.** niuse has seven Markdown stores, goals/plan/evidence/verify,
  a 10/10-reviewed opt-in digest, and a TUI that records turn metrics.
- **MCP.** `niucode-mcp` 0.1.5 is published (2026-10-23). ADR-0034 is accepted
  and niucode `ab50b75` implements the write path; niuse exposes
  `--allow-mcp-write`, ships an opt-in notes `create_note`, and the TUI PTY
  proves the approval preview plus deny/allow (M8 done).
- **Tests.** `./scripts/check.sh` and `./scripts/eval.sh` pass; the TUI suite
  covers approvals, session isolation, resume/fork pointers, memory prompts,
  and brief/report/steps composition.
- **Board.** 165 cards and 19 docs are generated and link-checked.
- **CI.** niucode `2b278fe` and niuse `ed54d6f` are green. niuse `c4282f3`
  passes local `check.sh`/`eval.sh`; GitHub CI is pending on a runner backlog,
  with no red run.

## Real evidence

- 2026-10-22: real `exec`, notes MCP, TUI, approved write, memory A/B, RSS
  disagreement, ten digest reviews.
- 2026-10-23: real TUI tasks read `DAY_TWO*.md`; the missing TUI `on_turn_end`
  was found and fixed, and the first metrics-backed TUI turn was recorded. A
  real model also called the notes `create_note` write tool with approval and
  created `stage-write-home.md`. The published 0.1.12 binary passed a TUI
  smoke test and a write-MCP smoke test.
- Dogfood: **2/10 days**, acceptance **100% (1/1)**, **3 distinct cited memory
  ids** (`m-f356`, `m-5c68`, `m-3d0b`), **1 disagreement**.

## Open gates

| Gate | State | What closes it |
|---|---|---|
| M2 dogfood | 2/10 days; citations and disagreement met | eight distinct real working days |
| M5-002 TUI | 2/10 days | eight more real TUI days |
| M3 publish | done: `niuse` 0.1.12 and `niucode-*` 0.1.6 published; [release page](release-0.1.12.md) | — |
| M8-002 mutating MCP | ADR-0034 proposed | human acceptance before any write tools |

## Decisions waiting on a human

1. Publish nothing further; 0.1.12/0.1.6 is live.
2. M2/M5 need distinct real working days.
3. Continue past round 79 only on an explicit request.

## Stage outcome

Five rounds (75–79) closed with the TUI command-composition test, the stage
report, and the last stale store count fixed. The implementation surface is
verified locally and on CI through `ed54d6f`; the product still needs eight
more real days, the MCP upload, and the mutating-MCP decision.

## Not claimed

- The ten-day dogfood gates are not met.
- Mutating MCP is not implemented.
- A resumed or forked session is live for the run, not the next launch
  ([ADR-0007](adr/0007-active-thread.md)).
