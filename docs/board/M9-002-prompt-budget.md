+++
id = "M9-002"
title = "Prompt-size regression budget"
status = "done"
milestone = "M9"
module = "prompt"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M9-001"]
docs = ["evals", "roadmap"]
+++

## Problem

Every turn pays for the injected prompt. Memory, goals, plans, loops, and
journal each have a budget, but no test proved the composed prompt stayed
bounded as a store filled up. A truncation regression would quietly raise cost
on every turn and could break the stable cache prefix.

## Scope

- A fat-store fixture: 200 memories, 7 goals, 6 plan steps, 20 promises,
  10 journal days, identity, and user.
- Assert total composed prompt ≤ 4,200 tokens and each section within its
  budget plus a small instruction allowance.
- Plan flood: 50 steps inject six and say the rest are omitted.
- Memory flood: 500 entries are dropped rather than appended.
- Run through `./scripts/eval.sh`.

## Acceptance

- [x] Fat prompt is under 4,200 estimated tokens.
- [x] Each injected section is within its budget plus slack.
- [x] A 50-step plan injects six steps and reports the omission.
- [x] 500 memories produce dropped entries, not a longer prompt.
- [x] `Composition.tokens` matches the shared estimator.
- [x] The eval runner includes the case.

## Notes

This is a token-count regression, not a provider bill. Prices and cache hits
still need real usage; `scripts/dogfood-trend.sh` watches tokens per turn.
