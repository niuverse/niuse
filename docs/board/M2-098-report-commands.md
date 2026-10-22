+++
id = "M2-098"
title = "Weekly report in REPL and TUI"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-097"]
docs = ["evals", "roadmap"]
+++

## Problem

`niuse report` existed only as a subcommand. The weekly review is most useful
where the work happens, without leaving the session.

## Scope

- Shared `report_text` used by CLI, REPL, and TUI.
- REPL `/report [days]`, default 7.
- TUI `/report`, default 7 and `cwd/docs/dogfood.md` when present.
- Add both to help/completion.

## Acceptance

- [x] REPL `/report 14` parses the day window.
- [x] TUI `/report` returns the same report text.
- [x] The dogfood file is picked up from `cwd/docs/dogfood.md`.
- [x] Commands appear in help/completion.
- [x] No second implementation of the report.

## Notes

The CLI remains the JSON surface; the in-session commands are text-only.
