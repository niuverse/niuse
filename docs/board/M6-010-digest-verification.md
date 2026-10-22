+++
id = "M6-010"
title = "Digest includes verification evidence"
status = "done"
milestone = "M6"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M6-009"]
docs = ["roadmap", "evals"]
+++

## Problem

The digest transcript was user/assistant prose only. A session that ended with a
recorded verification looked like a conversation with no evidence, so the digest
could not say whether a step passed or failed.

## Scope

- Include `verification` events as compact lines:
  `[verification] p-1 ok cargo test` / `FAILED`.
- Keep the command, verdict, and optional step; never include tool output.
- Count these lines against the same recent-item and character budgets.
- Test the transcript shape.

## Acceptance

- [x] A recorded verification appears in the transcript with step and verdict.
- [x] Failed verifications say `FAILED`.
- [x] Tool output is still excluded.
- [x] The recent-item limit applies to messages and verifications together.
- [x] Existing digest, gate, NONE, and secret tests pass.

## Notes

2026-10-22 real plan+verify run: the `[verification] p-1 ok grep ...` line
reached the transcript, but the digest kept the decision and open question
instead. That is acceptable: a weak file existence check is not the change.
Verification evidence is human-run and bounded at write time; including it is
not a tool-output leak.
