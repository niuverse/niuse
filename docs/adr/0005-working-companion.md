+++
id = "adr-0005"
title = "ADR-0005: A working companion, not only a memory layer"
kind = "adr"
status = "accepted"
order = 23
summary = "niuse stays terminal-first and file-based, but grows project awareness, proactive briefs, long-horizon goals, and delegation. Generic capabilities land in niucode first."
+++

## Status

Accepted · 2026-10-22. This revises the non-goals in [plan](../plan.md) §12 and D8
(TUI deferred). [ADR-0003](0003-kernel-product-boundary.md) still governs the
kernel/product split, and [ADR-0004](0004-renderer-deferred.md) still gates
removing the renderer.

## Context

The MVP proved the memory loop: identity, confirmed memory, automatic journal
and loops, soft expiry, secret scanning, and a resumable thread. The remaining
product gate is dogfood, not another store.

A companion that can only remember is a diary. The useful next step is to turn
remembered context into work: notice what is due, know which project is in
flight, plan, act through tools, delegate, verify, and report evidence. niucode
now exposes the primitives for that work: the library target, `TurnHooks`, the
TUI, `/plan`, `/goal`, subagents, skills, and the session-scoped extension host.

At the same time, "more powerful" cannot mean "a second framework". The
product boundary remains: niucode owns loop, tools, providers, sandbox, and the
event log; niuse owns identity, memory, product defaults, and the way work is
framed for one person.

## Decision

1. **The product is a working companion.** Memory is the substrate, not the
   deliverable. niuse may add project awareness, startup briefs, long-horizon
   goals, planning, delegation, and evidence capture when they make the
   companion more useful in real work.

2. **Generic capability goes upstream first.** A better loop, goal evaluator,
   subagent, provider, tool, permission model, or connector belongs in niucode
   and is consumed through the pinned API. niuse adds only personal semantics
   and product UX. It never forks kernel files.

3. **Terminal-first, no daemon by default.** Proactivity happens at session
   start, session end, and explicit commands. Background scheduling, push
   notifications, or a resident service require a later ADR with a concrete
   need; "always on" is not a default.

4. **Markdown and the event log stay the only durable truth.** No database, no
   vector index, no second session store. Derived views are rebuilt, not
   persisted as authority.

5. **Safety is part of the capability.** Approvals, sandbox policy, secret
   scanning, and least privilege apply to every new tool, connector, and
   delegated agent. A capability that cannot be bounded is not shipped.

6. **Every phase needs proof.** A phase exits on offline tests plus real use:
   a dogfood record, a cited past judgement, a completed goal, or a measured
   regression. A feature that only demos is not done.

## Consequences

**Better**

- The product can grow from "remembers" to "gets work done" without changing
  the kernel's identity.
- Existing niucode work (TUI, goals, subagents, skills) becomes reachable to the
  person instead of being rebuilt in niuse.
- The roadmap has explicit exits, so a long-running effort cannot drift into
  collecting features.

**Worse**

- The surface area grows and needs product judgement, not just code.
- More of niuse's value depends on the pinned niucode revision and its contract
  tests.
- Dogfood and evaluation become more important, because autonomy failures are
  worse than memory failures.

## Alternatives

- **Stay memory-only.** Rejected: it cannot use what it remembers, and every
  task still starts from zero.
- **Put personal semantics in niucode.** Rejected: it changes what the kernel
  is and couples two release cadences (ADR-0003).
- **Daemon-first companion.** Rejected for now: an operations project with a
  larger privacy surface, before the terminal loop is proven.
- **Adopt MCP and connectors immediately.** Deferred to M8: first make the
  existing harness usable end to end, then add external capability behind a
  trust boundary.
