+++
id = "M9-010"
title = "README command contract"
status = "done"
milestone = "M9"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M9-009"]
docs = ["roadmap", "plan"]
+++

## Problem

README command examples can drift from clap, and the REPL command list can name
a command that no longer exists. The docs test found `/steps` documented for the
REPL but missing from `SLASH_COMMANDS`.

## Scope

- Parse fenced `niuse <subcommand>` examples; assert clap knows each one.
- Parse the README's "Inside the REPL" list; assert every base command is in
  `SLASH_COMMANDS`.
- Skip prose lines, comments, flags, and feature-gated `tui`.
- Add `/steps` (list/add/done) to the REPL, sharing the CLI plan gate.

## Acceptance

- [x] Every README CLI example names a real subcommand.
- [x] Every README REPL command is registered.
- [x] `/steps` works in the REPL, not only the TUI.
- [x] The tests fail on an unknown command.
- [x] `check.sh` runs them with the normal test suite.

## Notes

This is a name-level contract, not an argument-level one. Flags can still drift;
a future test could compare `--help` text.
