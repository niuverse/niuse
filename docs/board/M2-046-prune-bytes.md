+++
id = "M2-046"
title = "Prune reports freed bytes"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-044"]
docs = ["architecture"]
+++

## Scope

- `mem prune --apply` reports how many bytes the deletion freed.
- Byte totals come from the two memory files before and after.
- The human-readable formatter is shared with the thread listing.
- `memory_bytes` has a unit test.

## Acceptance

- [x] Empty stores report zero bytes.
- [x] Adding one entry makes the total positive.
- [x] The formatter is reused instead of duplicated.
- [x] Failure to compute sizes cannot fail the prune.

## Notes

Deleting a handful of lines is not about disk pressure; the number is a sanity check that something actually changed. Board priorities only allow P0-P2, so this sits at P2 in practice.
