+++
id = "M6-004"
title = "On-demand memory recall"
status = "backlog"
milestone = "M6"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-22"
depends_on = ["M6-001"]
docs = ["roadmap", "adr-0001"]
+++

## Problem

Full injection is capped at about 2k tokens; entries outside the cap or past
expiry are invisible to the model even though they are still searchable by the
human. The companion cannot say "I remember a decision about this" unless it
happened to fit in the prompt.

## Scope

- Expose a bounded recall path the model can invoke during a turn, returning at
  most a few entries with ids, dates, and expiry state.
- Keep full injection as the default; recall is additive and read-only.
- Decide the seam: a niucode tool registered by niuse, or a product command the
  user invokes. Do not add personal schema to `niucode-core`.
- Budget the result and never dump the whole store into context.

## Acceptance

- [ ] A turn can retrieve an expired entry by keyword and cite its id.
- [ ] The result is capped and labelled with its freshness.
- [ ] The model cannot write, delete, or confirm memory through recall.
- [ ] Offline tests cover hit, miss, empty store, and budget truncation.
- [ ] The prompt size stays within the existing budget.

## Notes

Prefer a generic tool seam upstream in niucode; the memory semantics stay in
niuse.
