+++
id = "M2-073"
title = "Failed review edits keep the candidate"
status = "done"
milestone = "M2"
module = "bootstrap"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-067"]
docs = ["plan"]
+++

## Problem

In `niuse review`, the `e` path advanced past the candidate when the editor was abandoned, produced empty text, or triggered the secret scan. The candidate vanished from both the inbox and memory — neither accepted nor explicitly discarded.

## Fix

- Every failed edit now prints `保留到下次` and pushes the candidate back onto `remaining`.
- Accepting an edited candidate still removes it from the inbox and records `review_accept`.
- Manual check with a secret-bearing editor: inbox still contains the entry, summary says `剩余 1 条`, no memory file is written.

## Acceptance

- [x] Abandoned edit keeps the candidate.
- [x] Empty edit keeps the candidate.
- [x] Secret-bearing edit keeps the candidate.
- [x] A successful edit still accepts and clears it.

## Notes

The only actions that should destroy a candidate are an explicit accept or discard. An editor failure is not a decision.
