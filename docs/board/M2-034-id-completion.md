+++
id = "M2-034"
title = "Memory and loop id completion"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-033"]
docs = ["architecture"]
+++

## Scope

- After `/mem show`, `/mem edit`, `/mem confirm`, `/mem forget`, `/forget`, or `/done`, Tab completes ids from the stores.
- Memory candidates show a truncated text preview; loop candidates include only open loops.
- Ids are read at completion time, so a memory added earlier in the session is immediately completable.
- The logic is split into a pure `id_query`/`id_candidates` pair with a unit test.

## Acceptance

- [x] A memory id prefix resolves to the matching entry.
- [x] The replacement is the bare id; display carries the text preview.
- [x] Closed loops are excluded from `/done` completion.
- [x] Non-id commands still get command completion.

## Notes

This is the payoff of short ids: they are easy to type, and now easy to recall without leaving the prompt.
