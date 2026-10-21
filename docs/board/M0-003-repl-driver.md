+++
id = "M0-003"
title = "REPL and the niucode driver"
status = "done"
milestone = "M0"
module = "repl"
size = "L"
priority = "P0"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-002"]
docs = ["architecture"]
+++

## Scope

- Interactive REPL: streaming output, slash commands, Ctrl-C cancel, Ctrl-D exit.
- The driver opens or resumes `thread.jsonl` and rebuilds history with `project_history`.
- Each turn sets the working directory again, proving per-turn cwd on one session.
- Candidate confirmation: `y` / `n` / `e` / `s`; conflicts use `r` / `k` / `d`.

## Acceptance

- [x] A real provider run completes a read-only task and persists the session.
- [x] The same session reads in one directory, `/cd /tmp`, then reads in another; both succeed.
- [x] Confirming a candidate writes memory that `niuse mem list` can show.
- [x] Journal and loop blocks are written automatically.
- [x] The driver is about 200 lines and does not bypass the loop.

## Notes

`niucode-cli` is a binary with no library target, so its TUI and renderer cannot be reused. The MVP ships a minimal renderer and can swap later once upstream exposes a library or a hook.
