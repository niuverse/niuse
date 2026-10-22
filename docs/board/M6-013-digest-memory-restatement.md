+++
id = "M6-013"
title = "Gate ignores restated memory"
status = "done"
milestone = "M6"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M6-012"]
docs = ["roadmap", "evals"]
+++

## Problem

The tenth real digest was a memory Q&A: the assistant restated `m-3d0b`. The
gate said YES because the content was "new in the conversation", even though it
existed in the store before the session. The digest then repeated a preference,
which the digest prompt explicitly forbids.

## Scope

- Gate rule: restating injected memory is not new information; only a memory
  being changed, challenged, or used for a new decision counts.
- Re-run the same real session; expect NO with that reason.
- No new code beyond the gate prompt.

## Acceptance

- [x] Memory Q&A returns NO with a restatement reason.
- [x] The digest call is skipped; no journal, no metric.
- [x] A real decision session still returns YES (RSS).
- [x] Old-loop, trivial, Q&A, and failed/repair cases unchanged.
- [x] M6-003 reaches 10/10 real reviews.

## Notes

This is the final gate hardening for the ten-review sample. It stays
model-dependent; the reason line makes a wrong YES auditable.
