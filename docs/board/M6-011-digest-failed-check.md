+++
id = "M6-011"
title = "Digest captures failed-then-fixed verification"
status = "done"
milestone = "M6"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M6-010"]
docs = ["roadmap", "evals"]
+++

## Problem

A plan step failed its check, the file was repaired, and the re-check passed. The
gate still said NO: "read the file and report, then verification failed and
passed, no new decision". That is exactly the experience the journal should
keep.

## Scope

- Gate: a failed verification followed by a passing one is a turning point.
- Digest prompt: record the failure cause and repair, not only the final pass.
- No new events or hidden tool output; verification lines already carry the
  verdict and command.

## Acceptance

- [x] The same session now returns YES with that reason.
- [x] Digest produces a bullet for the initial failure.
- [x] Digest produces a bullet for the repair/final pass.
- [x] The final pass is not the only record.
- [x] Existing NO cases (old loop, Q&A, trivial) still pass.

## Notes

This does not make every failed check a journal entry: the failure must be
repaired and followed by a pass inside the session. Unresolved failures remain
covered by the "new unresolved issue" rule.
