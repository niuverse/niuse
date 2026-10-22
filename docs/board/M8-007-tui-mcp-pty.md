+++
id = "M8-007"
title = "TUI MCP mount PTY regression"
status = "done"
milestone = "M8"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
docs = ["roadmap", "evals"]
+++

## Problem

The TUI MCP path was wired but only the `exec` path had an end-to-end test.
A regression in niuse's TuiOptions or niucode's default manifest path would
only show up in manual use.

## Scope

- PTY test with `NIUCODE_HOME` pointing at a temp manifest.
- Replay provider calls `mcp__notes__search_notes`.
- Assert the startup `mcp: notes` line, the visible answer, and a successful
  tool result containing the note path in the append-only thread.
- No API key or network; reuses the notes pack and Python PTY harness.

## Acceptance

- [x] TUI mounts the notes server from `$NIUCODE_HOME/mcp.toml`.
- [x] Replay calls the namespaced MCP tool.
- [x] The thread records a successful `tool_result` with `note.md`.
- [x] The startup mount line is visible.
- [x] Runs under `cargo test -p niuse --features tui` and `check.sh`.

## Notes

This is a protocol regression, not a renderer certification. It closes the
M8-001 follow-up note.
