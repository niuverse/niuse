+++
id = "M10-005"
title = "Archive and start a new thread from web"
status = "done"
milestone = "M10"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-004"]
docs = ["roadmap"]
+++

## Problem

`niucode web` could create a fresh JSONL, but niuse disabled it: `/new` must
archive the active thread and recreate `thread.jsonl`, not create a side log.

## Scope

- Supply niucode's `new_session_provider`.
- Archive `thread.jsonl` into `threads/` via the existing driver.
- Create a new `thread.jsonl` with `SessionStart` + `Policy`.
- Use workspace-write / network-off / ask for the new thread header.
- Browser `/new` posts, shows the id, and reloads onto the new SSE path.
- Default niuse build remains niucode-free.

## Acceptance

- [x] The provider archives and recreates the active thread.
- [x] A unit test asserts the archive exists and the new file has a header.
- [x] `web` feature build/test/clippy pass.
- [x] `allow_new` is true only in the web build.
- [x] No new store or second index.

## Notes

The old page's SSE connection keeps tailing the archived file until reload. A
future session-switched SSE event can remove that window.
