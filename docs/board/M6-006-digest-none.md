+++
id = "M6-006"
title = "Digest NONE is a no-change"
status = "done"
milestone = "M6"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M6-003"]
docs = ["roadmap", "evals"]
+++

## Problem

The digest prompt tells the model to output `NONE` when nothing durable happened,
but the command treated that as an error ("no journal entries"). A healthy
no-change day looked like a failure, and an explicit signal was indistinguishable
from malformed chatter.

## Scope

- `NONE` (case-insensitive, trimmed) exits 0, prints no-change, writes nothing,
  and records no `digest` metric.
- Empty or prose-only output remains an error.
- Dry-run reports the same no-change.
- Tests cover both branches and the metric.

## Acceptance

- [x] `NONE` is a success with no journal write.
- [x] No digest metric is recorded for `NONE`.
- [x] Prose without bullets still fails.
- [x] Dry-run says there is nothing to write.
- [x] Secret refusal and five-bullet cap are unchanged.

## Notes

This addresses the activity-only caveat from M6-003: a short successful task can
legitimately have nothing to distill. It does not make the digest automatic.
