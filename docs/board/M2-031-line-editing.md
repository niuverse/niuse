+++
id = "M2-031"
title = "Line editing and persistent history"
status = "done"
milestone = "M2"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-002"]
docs = ["architecture"]
+++

## Scope

- The REPL uses `rustyline` when stdin is a terminal: cursor movement, word editing, Ctrl-A/E, and Up/Down history.
- History persists at `~/.niuse/history`, loaded at startup and saved on exit, mode `0600`.
- Piped or non-TTY stdin keeps the plain line reader, so scripts and tests are unaffected.
- Ctrl-C at the prompt clears the line; Ctrl-D exits. During a turn, Ctrl-C still cancels as before.

## Acceptance

- [x] A PTY session writes a private history file containing the entered lines.
- [x] Piped `/quit` still exits zero and does not create a history file.
- [x] History permissions are covered by a unit test.
- [x] Init failure falls back to the plain reader instead of aborting.

## Notes

A companion you talk to every day needs arrow keys. Keeping the piped path unchanged is what lets the integration tests keep exercising the real binary.
