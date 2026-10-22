+++
id = "M8-003"
title = "Read-only notes MCP pack"
status = "done"
milestone = "M8"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M8-002"]
docs = ["roadmap", "compat"]
+++

## Problem

MCP v1 was proven against a synthetic fixture, not a real personal source. The
connected-agent claim needs one pack that a person can point at real data and
one test that proves the boundary holds.

## Scope

- `packs/notes/server.py`: a standard-library stdio MCP server over `NOTES_DIR`.
- Read-only tools: `search_notes`, `read_note`, `list_notes`.
- Resolve paths before confinement; reject `..` and escaping symlinks.
- Cap reads and search hits; skip hidden files.
- Manifest example and setup notes in `packs/notes/README.md`.
- End-to-end test through `McpHost`.

## Acceptance

- [x] Search returns `file:line` hits from real Markdown/text notes.
- [x] `read_note` returns bounded UTF-8 content.
- [x] `../escape.md` is a tool error, not a file read.
- [x] The server has no write tool and no network dependency.
- [x] `McpHost` mounts it and the namespaced tools dispatch.
- [x] A setup example exists for `~/.niucode/mcp.toml`.

## Notes

This is the first real pack, not a general file manager. A mutating pack waits
for the approval design in ADR-0033's follow-up.
