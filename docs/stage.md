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
- **MCP.** `niucode-mcp` packages and passes `cargo publish --dry-run`; the
  release order includes it.
- **Tests.** `./scripts/check.sh` and `./scripts/eval.sh` pass; the TUI suite
  covers approvals, session isolation, resume/fork pointers, memory prompts,
  and brief/report/steps composition.
- **Board.** 161 cards and 18 docs are generated and link-checked.
- **CI.** niucode `2b278fe` and niuse `ed54d6f` are green. niuse `c4282f3`
  passes local `check.sh`/`eval.sh`; GitHub CI is pending on a runner backlog,
  with no red run.

## Real evidence

- 2026-10-22: real `exec`, notes MCP, TUI, approved write, memory A/B, RSS
  disagreement, ten digest reviews.
- 2026-10-23: real TUI tasks read `DAY_TWO*.md`; the missing TUI `on_turn_end`
  was found and fixed, and the first metrics-backed TUI turn was recorded.
- Dogfood: **2/10 days**, acceptance **100% (1/1)**, **3 distinct cited memory
  ids** (`m-f356`, `m-5c68`, `m-3d0b`), **1 disagreement**.

## Open gates

| Gate | State | What closes it |
|---|---|---|
| M2 dogfood | 2/10 days; citations and disagreement met | eight distinct real working days |
| M5-002 TUI | 2/10 days | eight more real TUI days |
| M3 publish | `niucode-mcp` release-ready | authorized crates.io upload, then `cargo package -p niuse` |
| M8-002 mutating MCP | ADR-0034 proposed | human acceptance before any write tools |

## Decisions waiting on a human

1. Publish `niucode-mcp` 0.1.5 (the token exists; no upload is authorized).
2. Accept or reject [ADR-0034](https://github.com/niuverse/niucode/blob/main/docs/adr/0034-mutating-mcp.md).
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
