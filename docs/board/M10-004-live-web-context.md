+++
id = "M10-004"
title = "Live product context for the web sidebar"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-003"]
docs = ["roadmap"]
+++

## Problem

`niuse web` sent one context snapshot at startup. After a turn changed memory,
goals, plan, loops, or metrics, the sidebar kept the old values until restart.

## Scope

- Extract `web_context(&Store)` from `run_web`.
- Pass it as niucode's `context_provider`; it is called per `/api/context`.
- Rebuild identity/user/goals/plan/memory/loops/report from the stores each time.
- Keep the prompt composition independent and unchanged.
- Fail soft to a minimal niuse brand if a store cannot be read.
- No new endpoint, store, or write path in niuse.

## Acceptance

- [x] `web_context` returns brand, sections, and report.
- [x] Unit test asserts at least seven sections and the report title.
- [x] `--features web` build/test/clippy pass.
- [x] Browser refresh is owned by niucode's client.
- [x] Default niuse build still does not link niucode.

## Notes

The provider re-reads Markdown stores on every request; report generation is
bounded to the first 18 lines. No caching is added at this size.
