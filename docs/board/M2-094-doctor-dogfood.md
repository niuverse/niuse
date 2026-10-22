+++
id = "M2-094"
title = "Dogfood progress in doctor"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-001"]
docs = ["dogfood", "roadmap"]
+++

## Problem

The M2 ten-day gate lived only in `docs/dogfood.md`. A user running the product
had no signal of how many real days were recorded, so the state was easy to
lose track of.

## Scope

- Count distinct local dates with `turn` metrics.
- `niuse doctor` reports `dogfood: N active day(s)`, warn under ten, ok at ten.
- `--json` includes the same check.
- No new store and no fabricated day: metrics count active days, not reviews.

## Acceptance

- [x] Two turns on one day count once.
- [x] Turns on two days count twice.
- [x] Non-turn metrics do not count.
- [x] Under ten is a warning; ten or more is ok.
- [x] The check is present in text and JSON.
- [x] The docs say it is a proxy, not the acceptance review.

## Notes

Ten active days do not close M2; the notes and acceptance trend still decide.
