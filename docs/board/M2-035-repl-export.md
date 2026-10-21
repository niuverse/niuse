+++
id = "M2-035"
title = "Export from the REPL"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-012"]
docs = ["architecture"]
+++

## Scope

- `/export` writes the current thread as Markdown without leaving the REPL.
- With no argument it lands at `~/.niuse/export-<thread>.md`; an argument is a path, resolved against the session's current `/cd` directory.
- The exported file keeps the existing `0600` mode and protocol-block stripping.
- The default and relative path rules have unit tests.

## Acceptance

- [x] `/export /tmp/x.md` wrote a private file from a live REPL.
- [x] The default name derives from the thread file stem.
- [x] Relative paths follow the runtime cwd, not the process start directory.
- [x] `/help` and completion list the command.

## Notes

Export is how a session becomes reviewable by a human. It should not require quitting the conversation first.
