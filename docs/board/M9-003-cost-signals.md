+++
id = "M9-003"
title = "Cost signals in stats"
status = "done"
milestone = "M9"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M9-002"]
docs = ["evals", "dogfood"]
+++

## Problem

`stats` reported raw input/output/cached totals, but not the two numbers that
explain a bill: how much of the input was cached, and how many tokens each turn
cost. A regression in either was invisible unless a human did the division.

## Scope

- `cache_hit_percent`: cached / input, clamped to 100, `null` without input.
- `tokens_per_turn`: (input + output) / turns, `null` without turns.
- JSON fields plus two text lines.
- Unit tests for the arithmetic and a CLI test for the JSON shape.

## Acceptance

- [x] 1,000 input + 500 cached reports 50% cache hit.
- [x] 1,000 input + 200 output + 1 turn reports 1,200 tokens/turn.
- [x] No input or no turns reports `null`, not zero.
- [x] Cached > input is clamped to 100.
- [x] Text output labels both numbers.
- [x] The CLI test pins the JSON keys.

## Notes

No price is guessed. The numbers are provider-agnostic; a future price table
can multiply them without changing this schema.
