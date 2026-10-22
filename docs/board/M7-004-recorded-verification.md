+++
id = "M7-004"
title = "Recorded verification command"
status = "done"
milestone = "M7"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M7-003"]
docs = ["roadmap", "evals"]
+++

## Problem

`/evidence` could show checks that happened to run during an agent turn, but a
human still had to scroll the transcript to prove a task after execution. There
was no first-class way to say "this is the command that decides", run it, and
keep the result in the thread.

## Scope

- `verification` event in niucode: command, ok, bounded output tail.
- `niuse verify -- <command>` runs it, prints the tail, appends the event, and
  exits with the command's status.
- `/verify <command>` in the REPL does the same without leaving the session.
- `/evidence` lists recorded verifications separately from agent-run checks.
- The event is ignored by model-context projection; it is evidence, not
  conversation.

## Acceptance

- [x] A passing command appends `ok: true` and exits 0.
- [x] A failing command appends `ok: false` and exits non-zero.
- [x] JSON output includes command, code, output tail, and thread path.
- [x] Output is capped at 4k characters on a char boundary.
- [x] `/evidence` reports recorded verifications.
- [x] The event round-trips through the session log.

## Notes

The human chooses the command; this does not infer what "verified" means for a
task and does not run through the agent's approval path. That is deliberate:
it is a human assertion with a reproducible command attached.
