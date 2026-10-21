+++
id = "adr-0002"
title = "ADR-0002: An identity layer, not a daemon"
kind = "adr"
status = "accepted"
order = 21
summary = "Identity is five Markdown stores, not prompt magic; companionship comes from memory, promises, and disagreement, not tone; what persists is the identity, not a process."
+++

## Status

Accepted · 2026-10-21. The identity layer and REPL are implemented; journal and loops are written automatically from in-band protocol blocks.

## Context

The goal is a personal companion, not a smarter tool. Two shapes exist in the ecosystem:

- **pi-memory style**: `IDENTITY.md` / `USER.md` / `MEMORY.md` / `daily/`, injected at session start. Lightweight, but only a memory extension.
- **pi-durable-assistant style**: a resident runtime, web UI, scheduled tasks, background subagents, a WeChat channel, and approval-gated memory. Heavy, and it turns a personal project into an operations project.

The cheap way to fake a companion is to give it a persona and a warm tone. That slides quickly into role-play: it talks a lot, remembers little, never disagrees, and does not improve your judgement.

## Decision

1. **Identity is files, not prompt magic.** `identity.md`, `user.md`, `memory/`, `journal/`, and `loops.md` are five persistent stores, all human-readable and editable. You can rewrite its values without touching code.
2. **You write the identity; it may only propose changes.** History can mine `user.md` and memory candidates, but it cannot mine who it is.
3. **Injection order is stable-to-volatile**: `identity → user → memory → loops → recent journal`. The system prompt is assembled once per session.
4. **Graded write thresholds**: journal and loops are automatic; memory requires confirmation; identity is human-only.
5. **No daemon.** No web UI, no WeChat, no scheduled push. It exists while the terminal is open and does not pretend to live in the background.
6. **Companionship rests on three things**: remembering your judgements, tracking its own promises, and speaking up when you are wrong. Not emotional performance, not pet names, not spontaneous sentiment.
7. **It keeps a right to disagree.** `identity.md` must state its judgement criteria and boundaries; disagreement comes with reasons, about the work rather than the person.
8. **The journal records change only.** Three to five lines covering decisions, turning points, open questions, and shifts in mood; no activity logs, no duplication of memory.

## Consequences

**Better:**

- The identity is fully auditable: one `git diff` shows what changed about its character.
- No operational burden: no servers, ports, certificates, or backup strategy.
- Cold start can borrow history, so the first two weeks do not feel like meeting a stranger.

**Worse:**

- It only exists while the terminal is open; it cannot remind you on a commute.
- Without push, the freshness of `loops` depends on you opening the terminal.
- Identity quality depends on whether you answer the eight questions; skip them and it degrades into a generic assistant.

**Accepted trade-off:** prove that "remembers, reliable, willing to disagree" is valuable before adding reach. Doing it the other way around produces a role-playing bot that interrupts you everywhere.

## Alternatives

| Option | Rejected because |
|---|---|
| Resident daemon plus push | About five times the work, and it turns a personal project into an operations project; reach can come later |
| A persona paragraph in the system prompt | Not auditable, not revertible, and it drifts on every startup |
| Let the model self-update its identity | It slowly drifts toward pleasing you, which destroys the willingness to disagree |
| Reuse pi-durable-assistant | A complete Node + SQLite runtime, incompatible with "built on niucode" |
| Rebuild the persona from memory each session | No shared history, nowhere for the journal to accumulate, no relationship to grow |
