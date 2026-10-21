+++
id = "M2-045"
title = "Readable thread titles"
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

- The first user message of a thread supplies a one-line title, truncated to 60 characters with an ellipsis.
- `threads` prints the title beneath the metrics line; `threads --json` adds `title`.
- The title is derived at read time; no new field is written to the session log.
- A unit test covers first-message selection and the empty case.

## Acceptance

- [x] The title comes from the first non-empty user message.
- [x] Later messages do not replace it.
- [x] JSON carries `title` as a string or null.
- [x] No event-log change.

## Notes

A list of timestamped files is not a memory of what you were doing. Deriving the title keeps the log untouched and still answers "which one was this".
