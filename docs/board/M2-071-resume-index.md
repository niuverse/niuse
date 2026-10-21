+++
id = "M2-071"
title = "Resume a thread by index"
status = "done"
milestone = "M2"
module = "repl"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-070"]
docs = ["architecture"]
+++

## Scope

- `threads` numbers its rows; the active thread is still marked with `*`.
- `/resume <n>` closes the current log, opens the n-th entry, and reloads history.
- The index is one-based and matches the listing order at read time.
- Resuming the current thread is a no-op with a message.
- `thread_at` has a unit test for 0, 1, 2, and out-of-range.

## Acceptance

- [x] `threads` shows `[1]`, `[2]`, …
- [x] `/resume 1` resolves the same path the listing shows.
- [x] The old log gets a `session_end` before switching.
- [x] Invalid or missing indexes do not leave the session.

## Notes

Numbers are computed per command, not stored; new archives can shift them, so the listing and the resume normally happen together.
