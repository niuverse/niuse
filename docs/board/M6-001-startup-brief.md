+++
id = "M6-001"
title = "Startup brief"
status = "done"
milestone = "M6"
module = "cli"
size = "S"
priority = "P0"
updated = "2026-10-22"
completed = "2026-10-22"
docs = ["roadmap", "status"]
+++

## Problem

The startup banner reports counts, but not what needs attention: an overdue
promise with its due date, an unchecked bootstrap queue, or memory that has
expired and should be pruned. The companion knows these facts and still waits to
be asked.

## Scope

- Build a pure `niuse-core` brief from the existing stores: memory totals, stale
  and expired counts, open and overdue loops, the last journal day, and the
  unchecked inbox count.
- Add `niuse brief [--json]`, `/brief` in the REPL and TUI, and a compact
  actionable block after the interactive banner.
- Keep it offline: no provider, no model call, no writes.
- Add `NIUSE_BRIEF=off` for people who want the old quiet banner.

## Acceptance

- [x] `niuse brief --json` reports counts, overdue loop ids/dates, and the last
      journal day.
- [x] The startup block lists at most three overdue loops and the review queue;
      it prints nothing when there is nothing actionable.
- [x] `/brief` works in the REPL and the TUI.
- [x] Unit tests cover empty, stale/expired, overdue, and inbox cases.
- [x] `NIUSE_BRIEF=off` suppresses only the automatic block.

## Notes

Derived view only: no new store, no new event type, no second source of truth.
