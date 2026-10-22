+++
id = "M9-007"
title = "Stable prompt prefix and protocol heading"
status = "done"
milestone = "M9"
module = "prompt"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M9-002"]
docs = ["evals", "roadmap"]
+++

## Problem

Cache-hit rate depends on the bytes before `## 今天` staying identical when only
the date, loops, or journal change. Nothing pinned that invariant, and the
protocol section repeated its own `## 记忆协议` heading because the renderer
already adds the title.

## Scope

- Test: volatile date/loops/journal do not change the stable prefix.
- Test: memory changes do change it on purpose.
- Test: goals and plan live inside the prefix.
- Remove the duplicate protocol heading; assert it appears once.

## Acceptance

- [x] Changing day, promise, and journal leaves the prefix byte-identical.
- [x] The full prompts still differ.
- [x] `记忆协议` appears before `今天`, loops, and journal.
- [x] `记忆协议` appears once.
- [x] A memory change invalidates the prefix.
- [x] The plan and goal are in the prefix.

## Notes

This is a structural cost regression, not a provider cache measurement. It
protects the boundary the provider needs; real hit rates still come from stats.
