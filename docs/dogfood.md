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

Or record it from the repo root:

```sh
niuse dogfood --file docs/dogfood.md --note "one concrete memory win, disagreement, or -"
# or the wrapper: ./scripts/dogfood.sh "note"
```

`niuse dogfood` reads today's `turn` / `propose` / `accept` metrics, replaces an
existing row for the same date, and only writes a new file private. It refuses
to write what looks like a secret. The wrapper calls the binary so there is one
implementation of the table update.

The script runs `niuse stats --days 1 --json`, uses the local calendar date, and inserts or replaces today's row in this table. Run it at the end of a real working day, not to fill a gap.

`niuse doctor` also reports `dogfood: N active day(s)` from the turn metrics. That is a convenience counter, not acceptance: only the rows and notes below can close M2.

`./scripts/dogfood-check.sh` validates this table: dates, counts, duplicates, future days, and the ten-day/60%/three-citations/one-disagreement criteria. Without `--complete` it only reports gaps; with `--complete` it exits non-zero until every criterion holds. The citations and disagreement counts are text heuristics, so a human still reviews the notes.

| date | turns | proposed | accepted | note |
|---|---|---|---|---|
| 2026-10-22 | 19 | 1 | 1 | real exec/MCP/TUI/write; A/B cited m-3d0b; disagreement dropped RSS; 10/10 digests, M6 closed; M5 TUI day 1/10 |
| 2026-10-23 | 4 | 0 | 0 | same-day retest TEST_ANYTIME_OK; cited m-f356 m-5c68; real MCP create_note approved; TUI day 2/10; 4 turns |

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

`scripts/dogfood-trend.sh` automates the look: it compares the last seven
active days with the previous seven and fails when acceptance drops by more
than 20 points (once both windows have at least three active days and five
proposals). `--strict` also fails on a token-per-turn blow-up or a shrinking
day count. `scripts/publish-niuse.sh` runs it before publishing and refuses on
a regression.

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

A fourth self-check chose `k` at the conflict prompt: both lessons remained, and `metrics.jsonl` recorded `keep:<id>`. A real bootstrap over two short niucode sessions extracted zero candidates, which is plausible for 375- and 2.1k-char sessions; the tool correctly left the inbox empty. A later `--profile-only` over three sessions wrote a `user.md` draft and explicitly said there was no evidence for red lines instead of inventing them.

A third self-check used two sessions on a fresh home: the second session's banner showed the memory injected, a contradictory lesson triggered the `[r] 替换 / [k] 并存 / [d] 丢弃` prompt with the old text quoted, and `r` left exactly one entry. `stats` counted 2 proposals / 2 accepted; `/cost` across the resumed thread reported 2 turns.

A sixth REPL check went `/new` → `/resume 2` → `/new`: the stale active log was archived instead of reopened, and two archives in the same millisecond got a `-1` suffix instead of overwriting each other.

A fifth, provider-free REPL check exercised `/new`, numbered `/threads`, `/resume 2`, and `/cost`: the archive moved to slot 2, resume switched the active marker, and `/cost` reported the archived thread's 2 turns from its own log.

This is a smoke test of the loop, not evidence about usefulness. The ten real days are still the gate.
