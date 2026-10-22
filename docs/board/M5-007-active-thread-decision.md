+++
id = "M5-007"
title = "Active-thread decision after resume"
status = "done"
milestone = "M5"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M5-006"]
docs = ["roadmap", "adr-0007"]
+++

## Problem

`/resume` and `/fork` updated the live pointer but not the next launch's active
thread. Leaving that ambiguous invited either a stale pointer file or silent
archive rewriting.

## Scope

- ADR-0007 decides: `thread.jsonl` is the only active line across restarts.
- `/resume` and `/fork` are session-scoped views, not promotions.
- `/new` and `/reset` are the only cross-restart active-line changes.
- Document the rule in architecture and status.

## Acceptance

- [x] A pointer file is explicitly rejected.
- [x] Archive promotion is explicitly rejected.
- [x] In-session hooks still follow `/resume` (M5-006).
- [x] `/new`/`/reset` archive and reset `thread.jsonl`.
- [x] Status and architecture say which command changes the next launch.

## Notes

This closes the open product decision from M5-006 without new code. If real
users need a chapter to become the main line, the explicit fallback is export or
manual file movement.
