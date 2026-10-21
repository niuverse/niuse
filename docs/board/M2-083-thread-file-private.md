+++
id = "M2-083"
title = "Thread logs are private from creation"
status = "done"
milestone = "M2"
module = "driver"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-003"]
docs = ["architecture"]
+++

## Problem

`open_thread` used `File::create` and then `set_private`. A new thread in a user-supplied `--thread` path was briefly `0644`, and an existing thread with loose permissions was never repaired on open.

## Fix

- New thread files are opened with `create_new` and mode `0600` on Unix.
- `set_private` now runs on every open, repairing an existing loose file.
- Unix tests cover creation mode and repair of a manually chmodded `0644` file.

## Acceptance

- [x] A new thread file is `0600` immediately.
- [x] An existing `0644` thread becomes `0600` on open.
- [x] Content and session start behavior are unchanged.
- [x] No new dependency.

## Notes

Session logs contain prompts and tool output; the file should never be world-readable, even for a moment.
