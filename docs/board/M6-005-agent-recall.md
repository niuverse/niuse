+++
id = "M6-005"
title = "Agent-invoked memory recall"
status = "done"
milestone = "M6"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
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

- [x] A turn retrieves a matching entry and receives its id and freshness in
      the tool result.
- [x] The tool cannot write, delete, or confirm memory.
- [x] The result is capped by the query limit and reuses unified recall.
- [x] Offline tests cover a hit, an empty query, and a full scripted turn.
- [x] The seam is generic and has a niucode contract test (M5-010).

## Notes

The tool is registered in the REPL, `exec`, and TUI paths through
`AgentConfig.extra_tools`; it is read-only and therefore needs no approval.
Memory is already secret-scanned at the write, so recall introduces no new write
surface. niucode caps tool output and compacts the conversation; the query limit
keeps the result small. The memory semantics stay in niuse.
