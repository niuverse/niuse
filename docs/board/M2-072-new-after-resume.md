+++
id = "M2-072"
title = "New after resume starts a fresh log"
status = "done"
milestone = "M2"
module = "repl"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-071"]
docs = ["architecture"]
+++

## Problem

After resuming an archive, `/new` archived the resumed thread but reopened the existing `thread.jsonl` instead of creating a fresh one, so the "new" line silently reused the old session start.

Two archives created within the same millisecond also shared a destination name, and `rename` would replace the earlier file.

## Fix

- `/new` archives the stale active log when the current thread is not `thread.jsonl`, then opens a genuinely fresh one.
- Archive names gain a `-1`, `-2` suffix until the path is free.
- `unique_archive_path` has a same-stamp unit test.
- End-to-end: `/new` → `/resume 2` → `/new` left two archives (`thread-…812` and `thread-…812-1`) and a fresh active log.

## Acceptance

- [x] The stale active log is archived, not reused.
- [x] Same-millisecond archives get distinct names.
- [x] The active log has exactly one new `session_start`.
- [x] The `/threads` listing shows active, original, and stale as separate rows.

## Notes

Both bugs only appear in the resume path; normal `/new` from the active thread already archived first.
