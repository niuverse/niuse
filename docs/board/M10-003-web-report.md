+++
id = "M10-003"
title = "Report panel in the web sidebar"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-002"]
docs = ["roadmap"]
+++

## Problem

The sidebar showed durable state but not whether the companion was actually
being used: turns, errors, acceptance, tokens, and dogfood days were only
available from `niuse report` in a terminal.

## Scope

- Call the existing read-only `commands::report_text(store, 7, None)`.
- Add a bounded `report · 7d` section (first 18 lines) to `/api/context`.
- Do not add a new metrics store or a write path.
- Fail soft: `report unavailable` if metrics cannot be read.

## Acceptance

- [x] The web feature builds and clippy passes.
- [x] The section comes from the same weekly report as the CLI.
- [x] Output is bounded to keep the sidebar usable.
- [x] Metrics remain append-only and unchanged.
- [x] No provider call is involved.

## Notes

The full report is still available from `niuse report`. Editing report inputs
from the browser is not planned for v1.
