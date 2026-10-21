+++
id = "M2-066"
title = "Token usage in stats"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-004", "M2-053"]
docs = ["architecture"]
+++

## Scope

- The renderer exposes accumulated usage for the run.
- REPL and `exec` append a `tokens` metric with `input,output,cached` after a run that used tokens.
- `stats` sums those values and prints `tokens  2k in · 400 out · 1.2k cached`; JSON exposes the three totals.
- Malformed token details are ignored, like other malformed metrics.

## Acceptance

- [x] Two synthetic records sum to 2000 / 400 / 1200.
- [x] Text uses compact counts; JSON uses raw numbers.
- [x] A run with zero usage writes no metric.
- [x] The summariser test covers the new kind.

## Notes

`/cost` answers "this thread"; `stats` now answers "this week". Both read recorded facts, not an in-memory counter.
