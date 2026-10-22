+++
id = "M8-005"
title = "MCP onboarding with bundled packs"
status = "done"
milestone = "M8"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M8-004"]
docs = ["roadmap", "evals"]
+++

## Problem

The packs existed, but onboarding meant reading source, finding absolute paths,
and hand-writing TOML. A personal agent that needs a manifest tutorial is not
ready for personal use.

## Scope

- `niuse mcp init --notes DIR --calendar FILE`.
- Write bundled server scripts to `~/.niuse/packs/` (or `--packs`).
- Write `~/.niucode/mcp.toml` (or `--config`) with read-only entries.
- Resolve an absolute python3 path, because the child environment is scrubbed.
- Refuse to overwrite an existing manifest without `--force`; no silent merge.
- Private dirs/files; scripts executable.
- End-to-end test mounts the generated manifest and calls both packs.

## Acceptance

- [x] One command writes scripts and a manifest.
- [x] The generated manifest validates and mounts.
- [x] Notes search and calendar search both work through it.
- [x] An existing manifest is refused without `--force`.
- [x] Python is stored as an absolute path.
- [x] `--config` / `--packs` make the test hermetic.

## Notes

No merge: replacing a trust file silently would be worse than asking for
`--force`. Mutating servers remain out of scope; the generated manifest has
only `read_only = true`.
