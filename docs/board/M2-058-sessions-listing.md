+++
id = "M2-058"
title = "List one-shot exec sessions"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-015"]
docs = ["architecture"]
+++

## Scope

- `niuse exec` writes its log under `~/.niuse/sessions/`; those logs were invisible in `threads`.
- New `niuse sessions [--json]` lists them with age, events, bytes, path, and a derived title.
- `threads` keeps its meaning: the active thread plus archived chapters, with the active marker.
- The directory scan is shared by both listings.

## Acceptance

- [x] A one-shot log appears with its first user message as title.
- [x] `active` is false for every one-shot.
- [x] JSON parses; text output is readable.
- [x] `threads` output is unchanged.

## Notes

`exec` was creating durable files with no discovery path. A separate command keeps "thread" semantics intact while making the logs findable.
