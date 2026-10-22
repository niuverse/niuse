+++
id = "M2-096"
title = "Dogfood row command"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-095"]
docs = ["dogfood", "evals"]
+++

## Problem

The daily row required copying numbers from `stats` into Markdown by hand. That
invites arithmetic mistakes and makes the ritual heavier than it needs to be.

## Scope

- `niuse dogfood --file PATH --note TEXT [--json]`.
- Read today's `turn` / `propose` / `accept` metrics.
- Replace the same-date row or append a new one; never duplicate.
- Refuse a note that looks like a secret.
- New files are private; existing repo files keep their mode.
- The wrapper script calls the binary so there is one update implementation.

## Acceptance

- [x] The row uses today's metrics, not values passed by the user.
- [x] A second run replaces the same date.
- [x] Other rows and surrounding text are preserved.
- [x] A missing table is an error, not a silent append.
- [x] `--json` reports the row and file.
- [x] The wrapper delegates to the binary.

## Notes

This writes a Markdown log, not a new durable store. The default path is
`~/.niuse/dogfood.md`; the project gate explicitly passes `docs/dogfood.md`.
