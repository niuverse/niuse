+++
id = "M2-012"
title = "Bootstrap inbox merge"
status = "done"
milestone = "M2"
module = "bootstrap"
size = "S"
priority = "P2"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-005"]
docs = ["plan"]
+++

## Scope

- Re-running `niuse bootstrap` merges into the existing queue instead of replacing it.
- Pending entries that have since been accepted into memory are dropped.
- Secret-bearing entries are dropped from both sides.
- New candidates are deduplicated against memory, the existing queue, and each other.

## Acceptance

- [x] Pending entries survive a second run; accepted ones disappear.
- [x] Duplicate candidates are not queued twice.
- [x] The merge rule has a unit test.
- [x] The CLI reports new versus total pending.

## Notes

Cold start is not a one-shot operation: sessions accumulate, and a second mining pass should add to the queue, not erase it.
