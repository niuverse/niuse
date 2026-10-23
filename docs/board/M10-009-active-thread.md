+++
id = "M10-009"
title = "Active thread in the web sidebar"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-006"]
docs = ["roadmap"]
+++

## Problem

The sidebar listed archived chapters but never said which thread was active.
After `/new` or a switch, the visible transcript had no companion label.

## Scope

- Add a `thread` context section showing `active · <file name>`.
- Read the active path from the store; tolerate missing name.
- Do not expose a full path or content.
- Keep the archived `threads` section unchanged.
- Test the section and count.

## Acceptance

- [x] Context contains a `thread` section.
- [x] It names `thread.jsonl` (or the active file).
- [x] Archived list remains unchanged.
- [x] At least nine sections are returned.
- [x] Web feature test/clippy pass.

## Notes

Switching still uses the archived `threads` buttons; the active section is
informational only.
