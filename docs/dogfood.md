+++
id = "dogfood"
title = "Dogfood log"
kind = "guide"
status = "living"
order = 40
summary = "Ten-day protocol: one row per day, numbers from stats --json, plus one concrete memory win or disagreement."
+++

## Why

M2-001 is a product acceptance gate, not a coding task. It needs ten days of real use, recorded while the context is fresh. The record is deliberately small: one row per day, two numbers, one sentence.

## Daily ritual (two minutes)

```sh
niuse stats --days 1 --json
```

Append one row below. `proposed` / `accepted` come from the JSON; `note` is a concrete memory win, a disagreement, or `-`. Do not reconstruct a missed day later; a gap is honest data.

| date | turns | proposed | accepted | note |
|---|---|---|---|---|
| YYYY-MM-DD | 0 | 0 | 0 | - |

## Rules

- A "memory win" means it cited a past judgement and saved a round of explanation, or avoided a repeated mistake. Name the entry id if possible.
- A "disagreement" means it pushed back and you changed your mind. Record what changed.
- If acceptance is zero for several days, stop and inspect the candidates instead of pushing through. Noise is a product bug.
- Keep the store under 50 entries. If it grows faster, the proposal bar is too low.

## Weekly check

```sh
niuse stats --days 7 --trend
```

Look for the direction, not the absolute number: acceptance should rise as the store matures, and the notes should become more specific.

## Done

Ten rows, acceptance at or above 60%, at least three cited judgements, at least one recorded disagreement. Then M2-001 can move to review, and the next milestone is decided from the notes rather than from the backlog.
