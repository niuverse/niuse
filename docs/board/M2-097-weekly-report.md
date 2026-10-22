+++
id = "M2-097"
title = "Weekly review report"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-096"]
docs = ["dogfood", "evals"]
+++

## Problem

The weekly review needed three commands: `stats`, `doctor`, and a manual read of
the dogfood table. The numbers were individually correct but never assembled.

## Scope

- `niuse report [--days N] [--dogfood PATH] [--json]`.
- Combine window turns/errors, acceptance, tokens/cache, open goals/loops/plan,
  and dogfood days/acceptance.
- Default the dogfood path to `docs/dogfood.md` when it exists.
- Read-only; no new store.

## Acceptance

- [x] Metrics produce turns, errors, acceptance, cache hit, tokens/turn.
- [x] Store counts produce open goals, promises (with overdue), plan steps.
- [x] Dogfood rows produce days and aggregate acceptance.
- [x] `docs/dogfood.md` is used when present.
- [x] `--json` is stable for scripts.
- [x] Text output is a compact Markdown-ready block.

## Notes

This is a review aid, not the M2 gate. The validator and human notes still
decide.
