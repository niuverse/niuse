+++
id = "M2-079"
title = "Editor temp files are private"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-003"]
docs = ["architecture"]
+++

## Problem

`$EDITOR` helpers wrote identity, user, memory, and candidate text to `/tmp/niuse-edit-<pid>.md` with default permissions — typically `0644`, readable by other local users, and containing secrets before the post-edit scan runs.

## Fix

- The temp file is opened with `create_new` and mode `0600` on Unix, so it is private from creation, not after a chmod race.
- The name includes the pid and nanosecond stamp; concurrent edits cannot share a path.
- Cleanup still removes the file after the editor exits; a failed write removes it too.
- A unit test checks content, uniqueness, and `0600`.

## Acceptance

- [x] Mode is `0600` at creation.
- [x] Two calls produce distinct paths.
- [x] The original text is written intact.
- [x] The file is removed on the normal path.

## Notes

The secret scan protects the store; it does not protect a world-readable scratch file that existed before the scan.
