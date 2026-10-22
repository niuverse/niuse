+++
id = "M5-004"
title = "Adopt the TUI's new session after /clear"
status = "done"
milestone = "M5"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M5-001"]
docs = ["roadmap"]
+++

## Problem

A PTY test for `/new` exposed a session-path mismatch. Niucode's TUI `/clear`
(or the new `/new` alias) calls `new_session`, which creates a timestamped file
in the session directory and ends the old `thread.jsonl`. Niuse's
`CompanionHooks` still holds `thread.jsonl`.

Consequences while the TUI runs:

- `/evidence` reads the old thread.
- `/verify` appends to the old thread.
- `plan done` checks the wrong thread.
- After exit, the next `niuse` launch resumes the old thread, not the one the
  user just worked in.

## Scope

- Let an embedder choose the new-session path, or receive `on_session_start`
  with the new path.
- Update `CompanionHooks::thread` and keep `thread.jsonl` as the active thread
  (archive the old one through niuse semantics).
- A PTY test must prove `/new` archives, clears context, and leaves product
  commands pointed at the new file.

## Acceptance

- [x] `/new` creates a niuse-understood active thread.
- [x] `/evidence` after `/new` reads the new session.
- [x] `/verify` and plan commands append to the new session.
- [x] The old thread is archived under `threads/`.
- [x] A PTY regression covers archive and context isolation.

## Notes

2026-10-22: fixed by niucode `323a158`. `TurnHooks::new_session_path` archives
`thread.jsonl`, returns the same active path, and `on_session_start` updates the
hooks. `/clear` and `/reset` both use it. The PTY test starts a second thread,
sees only `SECOND_ANSWER` in `thread.jsonl`, and finds `FIRST_ANSWER` in the
archived file.
