+++
id = "M2-043"
title = "Memory reviews are measured"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-004", "M2-008"]
docs = ["architecture"]
+++

## Scope

- `mem review` presents the oldest confirmed entries first and states how many are already expired.
- Each expired entry is marked individually.
- Keeping or deleting during review records `mem_review_accept` or `mem_review_delete`.
- `stats` and `stats --json` report `mem_reviews_kept` and `mem_reviews_deleted`.

## Acceptance

- [x] Expired entries are visible before a decision, not silent.
- [x] The two metric kinds are covered by the summariser test.
- [x] JSON includes the new fields.
- [x] A metric write failure is a warning, not a lost review.

## Notes

Review was the one memory path with no evidence trail. A ten-day dogfood cannot argue about pruning if the decisions are invisible.
