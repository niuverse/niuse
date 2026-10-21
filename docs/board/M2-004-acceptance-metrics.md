+++
id = "M2-004"
title = "Acceptance metrics and stats"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M2-003"]
docs = ["architecture", "plan"]
+++

## Scope

- `~/.niuse/metrics.jsonl`: tab-separated local counters, file `0600`.
- Record turn outcomes, candidate proposals and decisions, journal writes, loops, manual memories, and bootstrap review decisions.
- `niuse stats [--days N]` and `/stats` summarise the window; `--days 0` means all time.
- Acceptance rate is accepted over proposed; edited and replace/keep decisions count as accepted.

## Acceptance

- [x] Metrics append atomically and stay `0600`.
- [x] Parsing and summarisation have unit tests, including a window filter.
- [x] The CLI and REPL expose the summary.
- [x] No memory text is written to the metrics file.

## Notes

`exec` turns are counted in the same `turn` series. Candidates printed by `exec` are not counted as proposals, because no decision is collected.

This is the measurement M2-001 needs: without it, "acceptance rate ≥ 60%" is a feeling. The file is local-only and excluded from injection.
