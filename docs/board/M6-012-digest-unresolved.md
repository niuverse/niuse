+++
id = "M6-012"
title = "Digest keeps unresolved failures open"
status = "done"
milestone = "M6"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M6-011"]
docs = ["roadmap", "evals"]
+++

## Problem

The positive failed-then-fixed case is recorded, but the negative case matters
more: an unresolved failed check must never be summarized as success.

## Scope

- Real plan step, real model turn, one failed verification, no repair.
- Gate should say YES (new unresolved issue), digest should say failed and name
  the open choice, never "passed".
- No code change unless the output lies.

## Acceptance

- [x] Gate reason says the check failed and an open decision exists.
- [x] Digest bullet says `验证失败`, not passed.
- [x] The open choice (fix README vs adjust the step) is preserved.
- [x] No repair is invented.
- [x] M6-003 reaches 9/10 real reviews.

## Notes

This is the guardrail for M6-011: recording a repair must not train the digest
to assume every failure gets fixed. The two cases are now both evidenced.
