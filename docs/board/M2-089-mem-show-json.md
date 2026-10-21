+++
id = "M2-089"
title = "Single memory as JSON"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-025"]
docs = ["architecture"]
+++

## Scope

- `niuse mem show <id> --json` emits the same entry shape as `mem list --json`.
- The text view is unchanged.
- An end-to-end test adds an entry, reads its id from the list, and shows it back as JSON.

## Acceptance

- [x] JSON parses with id, text, and tags.
- [x] The shape matches the list output.
- [x] The text view is unchanged.
- [x] No storage change.

## Notes

Memory is now machine-readable at both the collection and single-entry level.
