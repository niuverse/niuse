+++
id = "M7-008"
title = "TUI plan and verify surface"
status = "done"
milestone = "M7"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M7-006"]
docs = ["roadmap", "evals"]
+++

## Problem

The REPL and CLI had `/verify` and `niuse plan`, but inside the TUI the plan
ledger was invisible and verification could not be tagged to a step without
leaving the session.

## Scope

- `/steps` lists open steps with verification state.
- `/steps add <text>` creates a step.
- `/steps done <N>` closes it through the same verification gate as the CLI.
- `/verify [p-N] <command>` runs and records a check, tagging the step.
- Commands appear in help and completion.

## Acceptance

- [x] `/steps add` returns a `p-N` label.
- [x] `/steps` shows `no recorded verification` until a check passes.
- [x] `/steps done` refuses without a passing tagged verification.
- [x] `/verify p-1 true` passes and `/steps done 1` closes the step.
- [x] The commands are listed for help/completion.
- [x] They share the CLI gate instead of a second implementation.

## Notes

`/plan` remains niucode's read-only plan-mode toggle; `/steps` is the niuse
ledger. Keeping them separate avoids overloading one command with two modes.
