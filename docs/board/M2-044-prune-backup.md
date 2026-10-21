+++
id = "M2-044"
title = "Back up memory before prune"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-008"]
docs = ["architecture"]
+++

## Scope

- `mem prune --apply` copies both memory files to `~/.niuse/backups/prune-<millis>/` before deleting anything.
- The backup directory is `0700`; copied files are `0600`.
- A failed copy aborts the prune; the store is not modified.
- The prune test asserts the deleted entry survives in the backup.

## Acceptance

- [x] Backup contains the deleted lesson text.
- [x] Directory mode is asserted on Unix.
- [x] Dry run writes no backup.
- [x] No new dependency.

## Notes

Hard delete with no undo is the one memory operation that cannot be reviewed after the fact. A timestamped copy is cheaper than a tombstone format.
