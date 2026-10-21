+++
id = "M2-015"
title = "Thread listing"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M2-013"]
docs = ["architecture"]
+++

## Scope

- `niuse threads` lists the active thread and every archive under `~/.niuse/threads/`.
- Each row shows age, event count, size, and the path relative to the niuse home.
- The active thread is marked with `*`, respecting `--thread`.
- Invalid sessions still appear with an event count of zero rather than hiding.

## Acceptance

- [x] The active thread and one archive are both listed.
- [x] The active marker follows `--thread` / the default path.
- [x] Age and size formatting have unit tests.
- [x] Listing is read-only and does not open sessions for append.

## Notes

`/new` made archives; without a listing they were only discoverable with `ls`. This closes the loop.
