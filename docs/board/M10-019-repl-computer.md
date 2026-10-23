+++
id = "M10-019"
title = "REPL computer policy and command"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-018"]
+++

## Problem

`--computer` was split across exec/TUI, and the default REPL had no way to
start, inspect, arm, or stop computer use. Text-only models also only failed
when the agent tried to observe.

## Scope

- Make `--computer` a global niuse flag shared by REPL, exec, and TUI.
- Build the platform runtime once in `App` and attach it to every REPL turn.
- Add `/computer`, `/computer arm`, `/computer stop` to the REPL with the same
  emergency-stop semantics as the TUI.
- Warn at startup when a configured model has no image capability.
- Keep off the default and preserve all non-computer commands.
- Parse tests for the global flag and existing REPL command coverage.

## Acceptance

- [x] `niuse --computer observe` starts a REPL with an armed runtime.
- [x] `/computer` reports policy/armed.
- [x] `/computer stop` disarms and releases input.
- [x] `/computer arm` re-arms the current policy.
- [x] REPL turns receive `config.computer`.
- [x] Text-only models warn before the first screenshot.
- [x] Tests/clippy pass.

## Notes

Web still uses a server-side policy later. The runtime is still constructed by
the kernel platform crates, not a product fork.
