+++
id = "M2-077"
title = "Tilde paths expand in cd and export"
status = "done"
milestone = "M2"
module = "repl"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-036"]
docs = ["architecture"]
+++

## Problem

Path completion resolved `~/` against `$HOME`, so Tab suggested it, but `/cd ~/project` joined the literal `~` to the current directory and failed. `/export ~/out.md` did the same.

## Fix

- One `expand_tilde` helper handles `~` and `~/…` using `$HOME`; unrelated paths pass through.
- `/cd` and `/export` both use it before relative-path joining.
- Unit test covers `~/out.md`, bare `~`, and a plain relative path.
- Manual check: `/cd ~` reports `cwd: /home/<user>`.

## Acceptance

- [x] Bare `~` becomes `$HOME`.
- [x] `~/…` becomes `$HOME/…`.
- [x] Plain paths are unchanged.
- [x] Completion and execution now agree.

## Notes

This was a mismatch between what completion advertised and what the command did. `$HOME` missing falls back to a literal path rather than panicking.
