+++
id = "M10-020"
title = "Cross-session Markdown memory evidence"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M6-001"]
+++

## Problem

The memory layers were tested separately: confirmation in the TUI, prompt
injection in unit tests, and persistence in store tests. No single case proved
that a confirmed Markdown entry survives a new store handle and changes a new
session's answer.

## Scope

- Add an offline end-to-end case in `personal_agent_eval`.
- Confirm a memory through `Store::add_memory`, drop the handle, reopen the
  same root, load Markdown, compose the system prompt, and run two independent
  sessions with separate JSONL logs.
- Assert the new sessions answer from the injected memory and are stable.
- Assert the confirmed entry lives under `memory/`.
- Reuse the existing TUI accept/discard PTY case for the confirmation barrier.

## Acceptance

- [x] Reopening the store returns the confirmed entry.
- [x] A new session's prompt contains it.
- [x] The scripted model changes its answer accordingly.
- [x] Two independent sessions give the same answer.
- [x] Markdown is the only persistence.
- [x] Tests/checks pass.

## Notes

This is evidence, not a new memory format. Candidate confirmation still happens
only through the human-facing prompt; the test calls the same confirmed write
path rather than bypassing it with a private file write.
