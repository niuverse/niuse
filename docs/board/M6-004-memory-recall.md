+++
id = "M6-004"
title = "Unified recall across stores"
status = "done"
milestone = "M6"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M6-001"]
docs = ["roadmap", "adr-0001"]
+++

## Problem

Full injection is capped at about 2k tokens; entries outside the cap or past
expiry are invisible to the model even though they are still in the stores. A
human can search memory one command at a time, but not memory, journal,
promises, and goals together, and the model has no honest way to ask for an old entry.

## Scope

- One bounded, read-only search over memory (including expired), journal
  bullets, loops, and goals.
- `niuse recall <query> [--limit N] [--json]`, `/recall` in the REPL and TUI.
- Rank with the existing lexical score, label freshness, and cap the result.
- Tell the model in the protocol that it may ask the user to run
  `/recall <关键词>` when it suspects a missing old memory; it must not guess.

## Acceptance

- [x] A query returns hits from all four stores with ids, dates, and detail.
- [x] Expired memory is included and labelled.
- [x] The result is capped and JSON is available for scripts.
- [x] Offline tests cover hit, miss, empty store, and the limit.
- [x] The model is told to request `/recall` instead of inventing a memory.

## Notes

This is the human-facing half. The model still cannot invoke recall by itself;
that needs a generic tool seam in niucode and is tracked as M6-005.
