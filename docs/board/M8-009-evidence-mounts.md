+++
id = "M8-009"
title = "Evidence shows MCP mounts"
status = "done"
milestone = "M8"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M8-008"]
docs = ["roadmap"]
+++

## Problem

ADR-0034 added `mcp_mount` events, but `niuse evidence` ignored them. A user
could see approvals and tool calls without seeing which servers were mounted,
which tools could write, or the config hash.

## Scope

- Parse `mcp_mount` into `Evidence.mounts`.
- Render `server · read-only` or `server · writes: … · hash`.
- Include mounts in the evidence header and in `is_empty`.
- Assert the TUI `/evidence` path after a real mutating mount.

## Acceptance

- [x] A mount-only thread is not reported empty.
- [x] The header counts mcp mounts.
- [x] The section shows server, write allowlist, and 16-hex hash.
- [x] The TUI PTY sees `writes:` and `mcp mount(s)` after `/evidence`.
- [x] Read-only mounts render as `read-only`.

## Notes

This is presentation only; the event log remains the source. The hash is the
same value niucode writes at mount, not a recomputation in niuse.
