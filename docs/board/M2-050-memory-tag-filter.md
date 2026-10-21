+++
id = "M2-050"
title = "Filter memory by tag"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-003"]
docs = ["architecture"]
+++

## Scope

- `mem list --tag rust` and `mem search <q> --tag rust` filter by tag, case-insensitively.
- `--tag` composes with `--kind`, `--all`, `--fresh`, and `--json`.
- The end-to-end CLI test adds two tagged entries and checks both filters.
- JSON output keeps the same shape.

## Acceptance

- [x] Listing by tag returns only matching entries.
- [x] Searching with a non-matching tag returns `[]`.
- [x] Case-insensitive tag comparison.
- [x] No storage change.

## Notes

Kind answers what a memory is; tags answer which subject it belongs to. Both are cheap filters over an in-memory list.
