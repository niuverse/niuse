+++
id = "M9-009"
title = "Board consistency check"
status = "done"
milestone = "M9"
module = "dashboard"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M9-008"]
docs = ["roadmap", "plan"]
+++

## Problem

The dashboard validates card references but not state contradictions. The
audit found M3 marked done while its distribution card was backlog, and M4
planned while its card was doing.

## Scope

- Parse milestone statuses and card frontmatter.
- Reject:
  - a done milestone with a non-done card;
  - a doing card in a planned milestone;
  - done without `completed`;
  - blocked without `blocked_reason`;
  - unknown milestones or statuses.
- Fix the two stale states: M3 is planned again; M4-001 is blocked on M5-002.
- Allow multiple active milestones after M2 and M5 gates legitimately overlap.
- Run live plus a deterministic self-test in `check.sh`.

## Acceptance

- [x] M3 no longer claims done while packaging is backlog.
- [x] M4-001 is blocked with a concrete reason, not doing in a planned phase.
- [x] Open card in a done milestone fails.
- [x] Doing card in a planned milestone fails.
- [x] Blocked without reason fails.
- [x] The live board passes.

## Notes

Multiple active milestones are allowed: M2 dogfood and M5 TUI dogfood run in
parallel. The earlier one-active rule was too strict. This checks state, not
judgement; a milestone can still be marked done too early with all cards done.
