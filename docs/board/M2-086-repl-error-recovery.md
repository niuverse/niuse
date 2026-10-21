+++
id = "M2-086"
title = "Slash-command errors do not end the session"
status = "done"
milestone = "M2"
module = "repl"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M0-003"]
docs = ["architecture"]
+++

## Problem

The run loop used `self.command(...)?`. A typo such as `/mem show nope`, an invalid `/cd`, or a failed `/export` propagated out of the loop, printed `Error:` and exited the whole REPL.

## Fix

- Command errors are printed in place and the loop continues.
- `Ok(Flow::Quit)` still exits; `Ok(Flow::Continue)` continues.
- Manual check: `/mem show nope` printed the error, `/status` still ran, `/quit` exited zero.

## Acceptance

- [x] A failed command prints an error and keeps the prompt.
- [x] The next command still executes.
- [x] `/quit` returns zero.
- [x] Turn errors are unchanged.

## Notes

Interactive tools should treat a bad command like a shell does: report and keep the prompt. Only explicit quit or a broken terminal should end the session.
