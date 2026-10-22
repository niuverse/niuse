+++
id = "M6-008"
title = "Digest gate ignores old-loop reminders"
status = "done"
milestone = "M6"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M6-007"]
docs = ["roadmap", "evals"]
+++

## Problem

The YES/NO gate counted a reminder of an old due loop as a "new unresolved
problem", so a routine session still passed and produced activity bullets.

## Scope

- Gate prompt distinguishes new/advanced issues from reminders of pre-existing
  items; an old loop reminder alone is NO.
- Real validation both ways: old-loop routine → NO; RSS disagreement → YES.
- No change to the digest call or write path.

## Acceptance

- [x] Routine session with only an old-loop reminder returns NO.
- [x] RSS decision session still returns YES and produces bullets.
- [x] The digest call count drops for old-reminder sessions.
- [x] Gate NO writes nothing and records no metric.
- [x] Existing gate and NONE tests pass.

## Notes

Prompt-only classification remains model-dependent. The real two-way check is
the evidence; a future hardening could return a one-line reason with the gate
verdict for audit.
