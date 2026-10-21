+++
id = "M2-062"
title = "Oversized promise does not hide the rest"
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

The loop section stopped at the first promise that did not fit the 300-token budget. A single long, overdue promise could therefore hide every shorter promise behind it.

## Fix

- Skip an oversized line and keep scanning; the budget is unchanged.
- A unit test puts a 2,000-character overdue promise before a short one and asserts the short one is injected.

## Acceptance

- [x] The short promise appears in the prompt.
- [x] The oversized one does not.
- [x] The budget and ordering are unchanged.
- [x] No storage or protocol change.

## Notes

Memory lines already used `continue`; loops used `break`. The asymmetry was accidental, not a policy.
