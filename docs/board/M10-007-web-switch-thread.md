+++
id = "M10-007"
title = "Switch archived threads from web"
status = "done"
milestone = "M10"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-006"]
docs = ["roadmap"]
+++

## Problem

The sidebar listed archived chapters but they were not selectable. Switching
required leaving the browser and moving files by hand.

## Scope

- Supply niucode's `switch_session_provider`.
- Validate the id (alphanumeric/`-`/`_`) and require the file to exist.
- Archive the active thread, move the selected chapter to `thread.jsonl`, and
  open it with the existing driver.
- Mark the `threads` context section with `kind: "threads"` so the client renders
  buttons.
- Confirm before switching; reload after success.
- No content injection, no arbitrary path input.

## Acceptance

- [x] A unit test moves an archived chapter back and preserves its content.
- [x] Invalid/missing ids fail without touching the active thread.
- [x] The active thread is archived before the switch.
- [x] `/threads` focuses the clickable section.
- [x] Web feature build/test/clippy pass.
- [x] No second store or index.

## Notes

Successful switches broadcast `session_switched`; every open tab reloads onto
the new path. Connections still finish against the old file until that reload.
