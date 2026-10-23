+++
id = "M8-008"
title = "Mount write-enabled MCP in niuse"
status = "done"
milestone = "M8"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M8-002"]
docs = ["roadmap"]
+++

## Problem

niucode now implements the accepted ADR-0034 write path (`write_tools`,
`--allow-mcp-write`, per-call approval, write lock), but niuse does not expose
the flag or ship a write-enabled pack. The product still mounts read-only
servers only.

## Scope

- Add `--allow-mcp-write` to `niuse exec` and `niuse tui`, passed to
  `niucode-mcp` mount.
- Refuse it with `--json`, `--approval never`, or a non-interactive run.
- Ship at least one write pack with an explicit `write_tools` allowlist.
- The TUI approval prompt shows the namespaced tool and bounded arguments.
- Integration test: deny writes nothing; allow writes once.

## Acceptance

- [x] niuse exposes the flag and forwards it without changing read-only defaults.
- [x] `--never` and a non-terminal mount read-only servers only.
- [x] A write pack lists every mutating tool; unlisted tools stay read-only.
- [x] TUI approval shows tool + args; denial leaves the store unchanged.
- [x] Offline test covers mount refusal and an approved write.

## Notes

niucode `ab50b75` is the implementation. This card is the niuse surface, not a
second design; ADR-0034 remains the source of truth.

2026-10-23: the flag and mount gating are in (`app.allow_mcp_write`, driver
`mount_mcp_from_with`, unit test). TUI forwards it to niucode. The notes pack
now has opt-in `create_note`, and `mcp_notes_write.rs` proves denial writes
nothing and approval writes once. 2026-10-23: `tui_mcp_write.rs` covers both outcomes. The PTY waits for the
namespaced tool and the `Arguments:` preview, sends `n` or `y`, and checks the
note file. Deny writes nothing; allow writes once. The card is done.
