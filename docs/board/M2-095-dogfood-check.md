+++
id = "M2-095"
title = "Dogfood log validator"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-094"]
docs = ["dogfood", "roadmap"]
+++

## Problem

The M2 close condition is a table in a Markdown file. Structural mistakes
(duplicate day, future date, accepted > proposed) and missing criteria could be
read as completion.

## Scope

- Parse the dogfood table; reject malformed cells, bad dates, duplicates,
  future days, negative counts, and accepted > proposed.
- Report days, aggregate acceptance, cited memory ids, and disagreements.
- Default mode reports gaps and exits 0 for an in-progress log.
- `--complete` exits non-zero until ten days, ≥60% acceptance, three cited
  ids, and one disagreement exist.
- Deterministic self-test with partial, complete, duplicate, over-count, and
  future fixtures.

## Acceptance

- [x] A one-row log reports `1/10` and exits 0.
- [x] The same log fails `--complete`.
- [x] A valid ten-row log passes `--complete`.
- [x] Duplicate, accepted > proposed, and future rows fail.
- [x] `check.sh` runs the live check and self-test.
- [x] The docs call the citation/disagreement counts heuristics.

## Notes

The validator cannot judge whether a day was real; it only stops arithmetic and
structure from lying. Human review remains the gate.
