+++
id = "M8-006"
title = "Safe MCP manifest merge"
status = "done"
milestone = "M8"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M8-005"]
docs = ["roadmap", "evals"]
+++

## Problem

`mcp init` refused any existing trust file unless `--force`, which replaced the
whole file. Adding the calendar pack after notes would have deleted a user's
other servers and comments.

## Scope

- Parse the existing manifest; if it is invalid, stop before writing.
- Merge by server name: add missing requested servers, keep existing ones.
- Preserve all existing text, including comments and formatting.
- `--force` still means replace.
- Report `added` / `kept` / `unchanged`.

## Acceptance

- [x] Notes-only manifest + calendar init produces both servers.
- [x] The original text and a custom comment survive the merge.
- [x] Re-running with both present is a no-op.
- [x] Invalid TOML fails and leaves the file byte-identical.
- [x] `--force` still replaces the file.
- [x] The generated merged manifest still mounts.

## Notes

Name collision is the merge key; a user server named `notes` is kept, not
overwritten. There is no reordering or comment-aware TOML rewrite.
