+++
id = "M6-005"
title = "Agent-invoked memory recall"
status = "backlog"
milestone = "M6"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-22"
depends_on = ["M6-004"]
docs = ["roadmap", "adr-0001"]
+++

## Problem

M6-004 gives the human `/recall`, and the prompt tells the model to ask for it.
That still costs a round trip and depends on the user being present. A stronger
companion can retrieve a missing old judgement inside the turn and cite it.

## Scope

- Add a generic, read-only tool seam in niucode that an embedder can register
  without forking the loop or adding personal schema to `niucode-core`.
- Register a bounded `memory_search` tool from niuse over the same store as
  `/recall`; results carry ids, dates, and freshness.
- Keep full injection as the default; recall is additive and cannot write,
  delete, or confirm memory.
- Budget the result and never dump the whole store into context.

## Acceptance

- [ ] A turn retrieves an expired entry by keyword and cites its id.
- [ ] The tool cannot write, delete, or confirm memory.
- [ ] The result is capped, labelled, and added to the prompt budget.
- [ ] Offline tests cover hit, miss, secret content, and budget truncation.
- [ ] The seam is generic and has a niucode contract test.

## Notes

Prefer a generic embedder tool or MCP resource over a niuse-only branch in the
kernel. The memory semantics stay in niuse.
