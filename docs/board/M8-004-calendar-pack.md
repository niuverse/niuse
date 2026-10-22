+++
id = "M8-004"
title = "Read-only calendar MCP pack"
status = "done"
milestone = "M8"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M8-003"]
docs = ["roadmap", "evals"]
+++

## Problem

The connected agent proved one personal source (notes). A second, different
shape of data is needed to show the MCP seam generalizes without a new kernel
feature.

## Scope

- `packs/calendar/server.py`: standard-library stdio MCP server over one ICS.
- `list_events(days)` and `search_events(query, limit)`.
- Unfold ICS lines, unescape text, parse date and time, sort, cap.
- `CALENDAR_ICS` required; `CALENDAR_TODAY` for deterministic tests.
- End-to-end test through `McpHost`.

## Acceptance

- [x] A timed event and an all-day event are listed.
- [x] An event outside the window is not listed.
- [x] Search matches description text and returns `no matches` cleanly.
- [x] The server has no write tool and no network dependency.
- [x] The pack mounts through the same read-only MCP host as notes.
- [x] A manifest example is documented.

## Notes

Single-file, read-only, no timezone conversion. Calendar subscriptions,
timezones, and writes need separate designs; writes are blocked on ADR-0034.
