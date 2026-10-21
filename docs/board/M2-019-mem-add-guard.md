+++
id = "M2-019"
title = "Manual memory duplicate guard"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-004"]
docs = ["adr-0001"]
+++

## Scope

- `niuse mem add` runs the same similarity rule as candidate writes.
- A match prints the similar ids and refuses unless `--force` is passed.
- `--force` now overrides both the secret scan and the duplicate check; the help text says so.
- Candidate and manual writes therefore share one conflict definition.

## Acceptance

- [x] A near-duplicate manual add is refused and lists the existing id.
- [x] `--force` adds it and the store grows to two entries.
- [x] The guard has a unit test.
- [x] Secret scanning still runs before the duplicate check.

## Notes

The confirmation flow already prevented duplicates; the manual path was the hole. Closing it keeps the store reviewable at 50 entries.
