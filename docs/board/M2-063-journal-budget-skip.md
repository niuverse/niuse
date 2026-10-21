+++
id = "M2-063"
title = "Oversized journal day does not hide older ones"
status = "done"
milestone = "M2"
module = "prompt"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M0-002"]
docs = ["architecture"]
+++

## Problem

The journal section stopped at the first day that did not fit the 600-token budget. One long day (a big debugging session) could hide every earlier day from the prompt.

## Fix

- Skip the oversized day and keep scanning, matching the memory and loop sections.
- A unit test puts a 4,000-character day before a short one and asserts the short one is injected.

## Acceptance

- [x] The older small day reaches the prompt.
- [x] The oversized day is omitted.
- [x] Budget and chronological ordering are unchanged.
- [x] No storage change.

## Notes

Recency is the ordering, not a reason to drop everything after one bad day. All three budgeted sections now use the same skip rule.
