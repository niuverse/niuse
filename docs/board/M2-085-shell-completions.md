+++
id = "M2-085"
title = "Shell completions"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M0-003"]
docs = ["architecture"]
+++

## Scope

- `niuse completions <shell>` prints a completion script for bash, zsh, fish, powershell, or elvish.
- The command returns before store discovery/init, so it works without a home and creates nothing.
- The CLI test asserts output content and that the home path is untouched.
- README lists the command.

## Acceptance

- [x] `completions bash` prints a `_niuse` function.
- [x] No `~/.niuse` is created.
- [x] `clap_complete` is the only new dependency.
- [x] Other commands are unchanged.

## Notes

Completions for the slash commands are provided by rustyline inside the REPL; this covers the outer CLI.
