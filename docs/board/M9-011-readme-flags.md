+++
id = "M9-011"
title = "README flag contract"
status = "done"
milestone = "M9"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M9-010"]
docs = ["roadmap", "plan"]
+++

## Problem

The name-level README test would pass even if a documented flag no longer
exists. Synopsis lines such as `loops list|add|done ... --overdue` also need
nested-subcommand awareness.

## Scope

- Parse fenced examples starting with `niuse <subcommand>`.
- Collect long flags from the command, its immediate subcommands, and root
  globals.
- Skip `tui`, flags-first lines, comments, and prose-only lines.
- Assert every documented `--flag` exists.

## Acceptance

- [x] A valid flag on a nested command passes (`--overdue`).
- [x] An unknown flag fails with the command and valid set.
- [x] Root globals are accepted.
- [x] Name-level and REPL contracts still pass.
- [x] The check runs in the normal test suite.

## Notes

This is still a smoke contract: flags from sibling subcommands are accepted so
synopsis lines can use `list|add|done`. Per-path flag validation is a future
refinement.
