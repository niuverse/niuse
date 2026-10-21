+++
id = "M2-006"
title = "Manual loops"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-001"]
docs = ["architecture"]
+++

## Scope

- `niuse loops add <text> [--due YYYY-MM-DD]` creates a promise without waiting for the model to emit a block.
- `/loop <text>` does the same inside the REPL.
- Due dates are validated; invalid dates are rejected before anything is written.
- Secret scanning applies to manual loops too.
- Creation is recorded in metrics.

## Acceptance

- [x] A manual loop persists with an optional due date.
- [x] An invalid due date writes nothing and exits non-zero.
- [x] The REPL path records the loop and a metric.
- [x] The CLI listing shows open and overdue entries.

## Notes

The model is not the only source of promises. This closes the gap where you know something needs tracking but the turn did not produce a loop block.
