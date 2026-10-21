+++
id = "M2-070"
title = "List threads and sessions inside the REPL"
status = "done"
milestone = "M2"
module = "repl"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-058"]
docs = ["architecture"]
+++

## Scope

- `/threads` prints the active thread and archived chapters without leaving the session.
- `/sessions` prints one-shot `exec` logs.
- Both reuse the CLI commands, so paths, titles, and age columns stay identical.
- `/help` groups them under the session section; completion lists both.

## Acceptance

- [x] The commands are part of the slash list and help coverage test.
- [x] Output is the same as `niuse threads` / `niuse sessions`.
- [x] No new state.
- [x] Piped sessions cannot reach them; this is TTY/REPL convenience.

## Notes

Leaving the conversation to inspect where you are is unnecessary ceremony. The commands read the same files the CLI reads.
