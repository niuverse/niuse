+++
id = "M2-099"
title = "Count distinct cited memories"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M2-001"]
docs = ["roadmap", "dogfood"]
+++

## Problem

`dogfood-check.py` counted one citation per **row**, not per distinct memory id.
Ten days that all said `m-abcd` would satisfy the "3 citations" gate, while one
day with three real ids would fail. The label already said "cited memory ids".

## Scope

- Collect `m-xxxx` ids across every note into a set.
- Fail `--complete` when fewer than three distinct ids are cited.
- Keep `cited_memory_ids` as a count; add `cited_ids` for the list.
- A repeated-id fixture must fail; the complete fixture uses three ids.

## Acceptance

- [x] Three distinct ids in one row count as three.
- [x] Ten rows repeating one id fail.
- [x] The self-test covers both cases.
- [x] The live table now reports 3 cited ids and only the 2/10 day gap.
- [x] M2-001's citation checkbox is satisfied.

## Notes

Today's real retrieval cited `m-f356`, `m-5c68`, and `m-3d0b`; the first task
cited two, the second listed all three. These are three different judgments,
not one id repeated.
