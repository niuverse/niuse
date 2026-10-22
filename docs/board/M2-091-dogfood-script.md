+++
id = "M2-091"
title = "One-command dogfood row"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-001"]
docs = ["dogfood"]
+++

## Scope

- `scripts/dogfood.sh "note"` reads `niuse stats --days 1 --json` and inserts today's row into `docs/dogfood.md`.
- The date comes from the local calendar, matching journal and memory.
- Running it twice in one day replaces the row instead of duplicating it; the `YYYY-MM-DD` placeholder is removed.
- `--file` supports a scratch copy; `NIUSE_BIN` / `NIUSE_HOME` select the binary and store.
- The guide explains that this records real use, not a reconstruction.

## Acceptance

- [x] First run inserts one row and removes the placeholder.
- [x] Second run replaces the same-day row; the count stays one.
- [x] Notes are pipe-escaped.
- [x] An end-to-end shell test in `scripts/check.sh` covers one row, replacement, and no duplicate.
- [x] The script fails clearly when `niuse` is missing or the table is not found.

## Notes

The command only formats numbers that already exist; it cannot make a day count. The ten real days are still the gate.
