+++
id = "adr-0006"
title = "ADR-0006: Goals are a sixth Markdown store"
kind = "adr"
status = "accepted"
order = 24
summary = "User outcomes get their own human-owned goals.md, injected after user and before memory; loops stay agent promises and goals never use progress percentages."
+++

## Status

Accepted · 2026-10-22. Extends the store list in [plan](../plan.md) §3.1 and
the six-store layout in [architecture](../architecture.md).

## Context

`loops.md` records what the companion promised to do. It is written
automatically when the model says it will follow up, and it is about the
companion's obligations. A user outcome such as "ship the personal-agent
roadmap this month" is a different thing: the human owns it, it spans sessions
and projects, and the companion should arrange work around it without silently
completing or rewriting it.

niucode's `/goal` is a session completion condition stored in the session event
log. It survives resume, but it does not survive a new thread and is not a
product-level intention. Reusing it would make a new chapter forget what the
human is trying to achieve.

The alternative of adding goals to `user.md` would force niuse to edit a
human-owned profile automatically, which breaks the ownership rule.

## Decision

1. **A sixth store.** `~/.niuse/goals.md`, same `0700`/`0600` rules and atomic
   writes as the other stores.
2. **Human-owned.** Only `niuse goals add|done` and `/goals` change goals. The
   model may read them but may not propose, complete, or rewrite them.
3. **Bounded.** At most seven open goals; adding beyond that is refused until
   one is completed or removed by hand.
4. **Injected after user and before memory**, budget 300 tokens. The section
   says explicitly that goals are maintained by the user.
5. **No deadlines and no progress percentages.** Deadlines belong to
   `loops.md`; percentages turn a companion into a task tracker. A goal is one
   sentence with an id, a creation date, and an open/done mark.
6. **Secrets are refused at the write**, because goals are injected every turn.

## Consequences

**Better**

- Long-horizon work survives a new thread, a new project, and a restart.
- The human/agent ownership boundary is visible in the file header and prompt.
- The store stays readable, reviewable, and cheap to inject.

**Worse**

- A seventh prose file to maintain and back up.
- Users who want sub-tasks or estimates will outgrow it; that is a signal to
  write a project plan, not to grow this store.

## Alternatives

- **Reuse `loops.md`.** Rejected: mixed ownership and two lifecycles in one
  file.
- **Reuse niucode's session `/goal`.** Rejected: it does not survive a new
  thread and belongs to the kernel's session state.
- **Auto-update `user.md`.** Rejected: it violates the human-owned profile.
- **A task tracker with progress.** Rejected: scope creep and prompt bloat.
