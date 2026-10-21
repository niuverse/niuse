+++
id = "M2-068"
title = "Rejected tags are suppressed for the session"
status = "done"
milestone = "M2"
module = "repl"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M0-003"]
docs = ["plan"]
+++

## Problem

The plan promises that after a memory rejection, the same tags are not proposed again in that session. The implementation discarded the candidate but kept offering the same tag, which is the confirmation fatigue the plan is trying to avoid.

## Fix

- `App` keeps a session-local set of rejected tags.
- Discarding a candidate (plain or conflict path) records its tags.
- A later candidate carrying any rejected tag is skipped and counted under `suppress`.
- The tag predicate is a pure function with a unit test.

## Acceptance

- [x] Case-insensitive tag matching.
- [x] Untagged candidates are not suppressed by tags.
- [x] Rejection from both the normal and conflict prompts records tags.
- [x] `s` still stops all proposals for the session.

## Notes

The set lives only in memory; a new session starts clean. This is a fatigue valve, not a permanent mute.
