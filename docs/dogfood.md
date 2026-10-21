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

## Self-check (does not count as a day)

2026-10-22 · synthetic write task in a scratch repo, `--sandbox workspace-write --approval ask`.

- The approval preview showed the expected one-line diff, `y` approved the write, and the file changed.
- Found: usage printed once per model step, so one prompt produced four usage lines; `/cost` counted steps as turns. Fixed in `M2-053`; a run now prints one `N steps · …` line and `/cost` reports `1 turn`.
- Found: `--approval ask` also prompts for read-only tools such as `bash cat`; the model recovered by using `read`. Not a bug, but the transcript makes the extra approval visible.
- `/export` wrote a `0600` Markdown file from the live REPL; `/stats` matched the one run.

A second self-check the same day covered two writes with the second approval denied: `a.txt` changed, `b.txt` stayed untouched, and the model reported the partial result. A scripted `$EDITOR` also exercised `mem review` → `e`: the text changed, the confirmation date refreshed, and `mem_review_edit` landed in `metrics.jsonl`.

A crafted offline inbox exercised `niuse review` without a provider: `y` accepted one candidate, `n` discarded the other, the inbox was rewritten empty, and `metrics.jsonl` recorded `review_accept` / `review_discard`.

A fourth self-check chose `k` at the conflict prompt: both lessons remained, and `metrics.jsonl` recorded `keep:<id>`. A real bootstrap over two short niucode sessions extracted zero candidates, which is plausible for 375- and 2.1k-char sessions; the tool correctly left the inbox empty.

A third self-check used two sessions on a fresh home: the second session's banner showed the memory injected, a contradictory lesson triggered the `[r] 替换 / [k] 并存 / [d] 丢弃` prompt with the old text quoted, and `r` left exactly one entry. `stats` counted 2 proposals / 2 accepted; `/cost` across the resumed thread reported 2 turns.

This is a smoke test of the loop, not evidence about usefulness. The ten real days are still the gate.
