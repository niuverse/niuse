+++
id = "M2-026"
title = "Memory kind filter"
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

- `niuse mem list --kind preference|lesson` filters the visible entries.
- `niuse mem search <query> --kind ...` applies the same filter before ranking.
- An unknown kind fails with the two accepted values.
- `--json` respects the filter; text output keeps the global counts.

## Acceptance

- [x] Listing lessons hides preferences.
- [x] Search ranks only the requested kind.
- [x] An invalid kind exits non-zero with a clear message.
- [x] Kind parsing has a unit test.

## Notes

At fifty entries the list is still readable, but "show me only decisions" is a real review need.
