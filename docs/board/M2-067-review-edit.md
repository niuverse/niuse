+++
id = "M2-067"
title = "Bootstrap review supports edit"
status = "done"
milestone = "M2"
module = "bootstrap"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-005"]
docs = ["plan"]
+++

## Problem

The plan's cold-start section says `niuse review` supports `y` accept, `n` discard, and `e` edit. The implementation only had `y`, `n`, `s`, and `q`; an almost-right candidate had to be accepted or discarded, not fixed.

## Fix

- Add `[e] 编辑`: open `$EDITOR`, refuse empty or unchanged results, scan for secrets, then accept the edited text.
- The accepted entry keeps the source session id; `review_accept` is recorded.
- Prompt, plan, and behavior now agree.

## Acceptance

- [x] `e` with a scripted editor changed the candidate text before accepting.
- [x] The edited entry kept the `style` tag and `s1` source.
- [x] Empty/secret edits are refused without writing.
- [x] `n` still discards and records `review_discard`.

## Notes

This was found by comparing the plan with the implementation, not by a failing test. The plan is the spec for the cold-start ritual.
