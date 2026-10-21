+++
id = "M2-087"
title = "Show only overdue promises"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-006"]
docs = ["architecture"]
+++

## Scope

- `niuse loops list --overdue` keeps only open promises whose due date is before today.
- JSON carries the same filter; the header says `N 条逾期` instead of the unfiltered totals.
- Works with `--all` for consistency; done promises are never overdue.
- End-to-end test adds one late and one future promise and asserts only the late one appears.

## Acceptance

- [x] Only overdue loops are listed.
- [x] JSON `overdue` is true for the result.
- [x] The header does not claim three open promises when none are shown.
- [x] No filter keeps the old output.

## Notes

The daily question is "what is late", not "what exists". The unfiltered list remains the default.
