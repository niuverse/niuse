+++
id = "M2-037"
title = "Multi-line input with backslash continuation"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-031"]
docs = ["architecture"]
+++

## Scope

- On a terminal, a line ending with `\` keeps the editor open instead of submitting.
- Continuation lines are joined with a newline and trailing whitespace is trimmed per line.
- The joined prompt is one history entry and one user message.
- Piped input is unchanged, so scripts and tests still submit per line.

## Acceptance

- [x] `hello \` + `there` produced a single `user_message` with `hello\nthere`.
- [x] The join helper has a unit test.
- [x] The continuation hint appears in `/help`.
- [x] No extra dependency.

## Notes

This is for pasted instructions and multi-paragraph context. It is deliberately a visible backslash, not an invisible mode.
