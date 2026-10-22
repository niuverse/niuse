+++
id = "M7-002"
title = "Read-only plan mode in the REPL"
status = "done"
milestone = "M7"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M7-001"]
docs = ["roadmap", "plan"]
+++

## Problem

niucode's TUI has `/plan`, but the default REPL does not. Before a multi-step
change, the companion should be able to investigate and propose a plan without
being able to write files or run mutating commands. The TUI path should not be
required for a safe planning pass.

## Scope

- `--plan` starts a session in plan mode; `/plan [on|off]` toggles it.
- Plan mode sets `AgentConfig.plan_mode` and narrows `allowed_tools` to
  `read`, `grep`, and `glob`.
- Protocol writes (memory, loop, journal) are ignored while planning.
- The banner shows `· plan` so the restricted state is visible.
- `exec --plan` gets the same treatment.

## Acceptance

- [x] `/plan on` narrows the tool allowlist and turns on the plan prompt.
- [x] `/plan off` restores the normal tool surface.
- [x] Protocol blocks are not written during a plan turn.
- [x] `--plan` starts the REPL and `exec` in plan mode.
- [x] The TUI receives `TuiOptions.plan` from the same flag.

## Notes

The prompt instruction comes from niucode; niuse only selects the mode and the
allowlist. No new store and no second approval path.
