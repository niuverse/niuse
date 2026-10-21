+++
id = "M2-016"
title = "Expired memory pruning"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-004"]
docs = ["adr-0001"]
+++

## Scope

- `niuse mem prune` lists entries older than 90 days and changes nothing.
- `niuse mem prune --apply` removes exactly those entries and records a metric.
- Fresh and stale-but-valid entries are untouched.
- Deletion is explicit; there is no implicit expiry write path.

## Acceptance

- [x] Dry run lists the expired entry and leaves the store unchanged.
- [x] `--apply` removes only expired entries; a fresh lesson survives.
- [x] The removal path has a unit test.
- [x] The metric records how many entries were pruned.

## Notes

Soft expiry means expired entries already leave the prompt. Pruning is housekeeping for the file, not a behaviour change; that is why it is manual.
