+++
id = "M8-008"
title = "Mount write-enabled MCP in niuse"
status = "doing"
milestone = "M8"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-23"
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
- [ ] A write pack lists every mutating tool; unlisted tools stay read-only.
- [ ] TUI approval shows tool + args; denial leaves the store unchanged.
- [ ] Offline test covers mount refusal and an approved write.

## Notes

niucode `ab50b75` is the implementation. This card is the niuse surface, not a
second design; ADR-0034 remains the source of truth.

2026-10-23: the flag and mount gating are in (`app.allow_mcp_write`, driver
`mount_mcp_from_with`, unit test). TUI forwards it to niucode. Still open: a
write-enabled pack and the end-to-end deny/allow test.
