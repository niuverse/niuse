+++
id = "M10-006"
title = "Archived threads in the web sidebar"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-005"]
docs = ["roadmap"]
+++

## Problem

`niuse web` could archive a thread on `/new`, but the archive disappeared from
the product surface. There was no way to see what chapters existed.

## Scope

- Read `threads/*.jsonl` from the store.
- Sort by modified time, newest first; show up to eight stems.
- Add a `threads` context section; tolerate a missing directory.
- Use niucode's `/threads` command to focus it.
- No switching, reading, or deletion from the browser.

## Acceptance

- [x] The context contains a `threads` section.
- [x] Missing archive directory yields an empty section.
- [x] Ordering is newest-first; output is bounded.
- [x] `/threads` focuses the section locally.
- [x] Test asserts the section and section count.
- [x] No store/index change.

## Notes

This was visibility only. **M10-007** adds the allowlisted switch protocol.
