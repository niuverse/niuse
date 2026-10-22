+++
id = "M8-002"
title = "Mount MCP in the REPL and exec"
status = "done"
milestone = "M8"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
docs = ["roadmap", "compat"]
+++

## Problem

niucode mounted read-only MCP servers in `exec` and the TUI, but niuse's own
REPL and `exec` build `AgentConfig` directly. Users could use MCP tools in
`niuse tui` but not in the default front end.

## Scope

- Mount through `niucode-mcp::McpHost` from `NIUSE_MCP_CONFIG` or
  `~/.niucode/mcp.toml`.
- Attach the host to every turn's `AgentConfig`, not just the TUI's.
- Shut the host down at REPL/exec exit.
- A broken manifest warns and disables MCP for that run; it does not fail the
  session.

## Acceptance

- [x] The REPL and `exec` set `AgentConfig.mcp_host`.
- [x] `NIUSE_MCP_CONFIG` overrides the default path.
- [x] Host shutdown runs on normal exit.
- [x] An offline test mounts the fixture server from a manifest and calls a
      namespaced tool.
- [x] `./scripts/check.sh` is green.

## Notes

The mount is read-only by construction; secrets never enter the child
environment unless the manifest names them.
