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
  - multiple active milestones;
  - a done milestone with a non-done card;
  - a doing card in a planned milestone;
  - done without `completed`;
  - blocked without `blocked_reason`;
  - unknown milestones or statuses.
- Fix the two stale states: M3 is planned again; M4-001 is blocked on M5-002.
- Run live plus a deterministic self-test in `check.sh`.

## Acceptance

- [x] M3 no longer claims done while packaging is backlog.
- [x] M4-001 is blocked with a concrete reason, not doing in a planned phase.
- [x] Open card in a done milestone fails.
- [x] Doing card in a planned milestone fails.
- [x] Blocked without reason fails.
- [x] The live board passes.

## Notes

This checks state, not judgement. A milestone can still be marked done too early
with all cards done; only evidence and review catch that.
