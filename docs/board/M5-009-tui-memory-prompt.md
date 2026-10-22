+++
id = "M5-009"
title = "TUI memory prompt PTY"
status = "done"
milestone = "M5"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M5-008"]
docs = ["roadmap", "evals"]
+++

## Problem

The after-turn memory prompt was unit-tested through `on_answer` but never
driven through the terminal. It uses blocking stdin while the TUI owns the
terminal, so key mapping or read behavior could differ from the REPL.

## Scope

- Replay a turn containing a `<<niuse:memory>>` block.
- Accept with `1`; the candidate must appear in `memory/`.
- Discard with `3`; the candidate must not appear.
- Assert the visible answer and the prompt question appear first.

## Acceptance

- [x] `VISIBLE_ANSWER` renders.
- [x] The `new memory` prompt and options render.
- [x] `1` stores the candidate.
- [x] `3` does not store it.
- [x] The edit path is not automated (it would launch `$EDITOR`).

## Notes

The prompt reads terminal input line-by-line; `\r` submits correctly in the
PTY. The edit option remains covered only by unit tests because it opens an
external editor.
