+++
id = "M2-025"
title = "Machine-readable memory output"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M2-003"]
docs = ["architecture"]
+++

## Scope

- `niuse mem list --json` emits the visible entries as a JSON array.
- `niuse mem search <query> --json` emits ranked matches with the same shape.
- Fields: id, kind, tags, created, confirmed, age_days, expired, source, text.
- Expiry state is computed at read time; the file format is unchanged.

## Acceptance

- [x] Both commands emit valid JSON that parses.
- [x] `expired` and `age_days` reflect the current date.
- [x] Serialization has a unit test.
- [x] Text output is unchanged when the flag is absent.

## Notes

Markdown stays the storage format. JSON is an output view for scripts and dogfood, not a second store.
