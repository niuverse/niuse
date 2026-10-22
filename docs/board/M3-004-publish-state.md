+++
id = "M3-004"
title = "Publish-state documentation"
status = "done"
milestone = "M3"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M3-001"]
docs = ["distribution", "roadmap"]
+++

## Problem

Status and distribution disagreed about the release state: status claimed
`cargo package -p niuse` succeeded and mentioned 0.1.3/0.1.6, while distribution
said packaging was blocked. The real state was mixed: 0.1.11 was published, but
the new `niucode-mcp` dependency was not.

## Scope

- Check crates.io for `niuse`, `niuse-core`, `niucode-*`, `niucode-mcp`.
- Run `cargo package -p niuse --no-verify` against the registry.
- Correct the seven-store count in the status snapshot.
- Record the exact blocker: `niucode-mcp` is unpublished.

## Acceptance

- [x] Status says seven stores, not six.
- [x] Status no longer claims packaging succeeds.
- [x] Distribution names the real current blocker.
- [x] M3-001 remains backlog with the same blocker.
- [x] No version is bumped or published in this card.

## Notes

Historical 0.1.3/0.1.6 install checks remain true for those versions; they are no
longer presented as the current state.
