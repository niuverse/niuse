+++
id = "M2-081"
title = "Export files are private at creation"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-012"]
docs = ["architecture"]
+++

## Problem

`write_export` used `fs::write` and chmodded to `0600` afterwards. A new export in a shared directory was readable for a short window, and exports can contain anything the session contained.

## Fix

- The file is opened with `create`/`truncate` and mode `0600` on Unix, so a new file is private from the first byte.
- The post-write chmod stays as a repair for pre-existing world-readable files.
- Existing export tests still assert `0600` on the result.

## Acceptance

- [x] New exports are created `0600`.
- [x] Existing files are still repaired to `0600`.
- [x] Content and truncation behavior are unchanged.
- [x] No new dependency.

## Notes

The same pattern as the store and editor fixes: create private, do not create then repair.
