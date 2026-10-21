+++
id = "M0-001"
title = "Five stores and atomic storage"
status = "done"
milestone = "M0"
module = "stores"
size = "L"
priority = "P0"
updated = "2026-10-21"
completed = "2026-10-21"
docs = ["architecture", "adr-0001", "adr-0002"]
+++

## Scope

- Five persistent stores: `identity.md`, `user.md`, `memory/`, `journal/`, `loops.md`.
- Plain Markdown; directories `0700`, files `0600`; write to a temporary file, then rename.
- Memory split into `preferences.md` / `lessons.md`; one journal file per month, merged by day.
- Civil dates computed in-tree, no timezone dependency.

## Acceptance

- [x] Round-trip tests for all five stores.
- [x] Same-day journal appends merge; different days stay ordered.
- [x] Permissions verified as `0700` / `0600`.
- [x] Missing `identity.md` returns `None` instead of a fabricated empty string.

## Notes

Markdown is the only format: for a single user under 200 entries, files are the best query engine. Saving rewrites the layout, so hand-written comments may disappear; that is the accepted cost.
