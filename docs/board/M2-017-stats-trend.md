+++
id = "M2-017"
title = "Stats daily trend"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M2-004"]
docs = ["architecture"]
+++

## Scope

- `niuse stats --trend` adds a per-day breakdown for the selected window.
- Days are local calendar dates; each row shows turns, accepted/proposed, and acceptance.
- At most the last fourteen active days are printed.
- `/stats` keeps the summary view; the flag is CLI-only for now.

## Acceptance

- [x] Metrics on two local days produce two rows.
- [x] Acceptance is `-` on days without proposals.
- [x] Grouping has a unit test with synthetic timestamps.
- [x] A real metrics file renders without panics.

## Notes

The dogfood question is not "what is the total" but "is it getting better"; a daily view makes that visible.
