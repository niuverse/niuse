+++
id = "M10-014"
title = "Final repository hygiene"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-013"]
docs = ["roadmap"]
+++

## Problem

A long series of test/debug edits can leave temp paths, formatting drift, or
untracked files before the final round.

## Scope

- Check both repos' full status, format, debug-path leftovers, TODO markers on
  the changed surface, and pin match.
- Record heads and non-checks.
- Do not change code.

## Acceptance

- [x] Both working trees clean including untracked files.
- [x] Both format checks pass.
- [x] No debug path leftovers.
- [x] Pin matches `23e23af`.
- [x] Evidence records what is not covered.

## Notes

Registry publication and the dogfood calendar remain external gates.
