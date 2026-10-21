+++
id = "M2-084"
title = "Metrics file is private at creation"
status = "done"
milestone = "M2"
module = "stores"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-004"]
docs = ["architecture"]
+++

## Problem

`append_metric` opened `metrics.jsonl` without a creation mode and chmodded to `0600` afterwards. It was the last writer using the create-then-repair pattern.

## Fix

- The open options set mode `0600` on Unix; the existing chmod remains as a repair for old files.
- The existing metrics test continues to assert the final mode and content order.

## Acceptance

- [x] New metric files are `0600` at creation.
- [x] Existing loose files are repaired.
- [x] Append order and sanitization are unchanged.
- [x] All store writers now use create-private-then-write.

## Notes

With store, editor, export, thread, and metrics fixed, no production writer creates a readable file before setting permissions.
