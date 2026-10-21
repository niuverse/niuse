+++
id = "adr-0001"
title = "ADR-0001: A global memory layer"
kind = "adr"
status = "accepted"
order = 20
summary = "Memory is a small global store, not a session index: plain Markdown, full injection, soft expiry, confirmation before writing; project knowledge stays in AGENTS.md."
+++

## Status

Accepted · 2026-10-21. The W1 spike passed; the MVP is implemented in `crates/niuse-core` (`store`, `policy`, `prompt`, `candidate`).

## Context

niucode already has `AGENTS.md`, skills, session resume, compaction, and fork, but it deliberately does not provide memory across sessions or projects (the roadmap lists it as deferred). The result: every new project starts by repeating preferences, decisions, and old pitfalls.

The obvious temptation is to index every session and retrieve from it. That direction has three concrete problems:

1. Code facts are one `grep` away; indexing duplicates the repository.
2. Retrieval quality needs an evaluation set, and niuse does not have one.
3. Vectorising turns "reviewable" into "opaque": a wrong memory cannot be spotted at a glance.

## Decision

1. **Memory is global, not per-session.** It stores only what still holds across projects; project-level knowledge stays in the repository's `AGENTS.md`, and niuse never writes there automatically.
2. **Three kinds only**: preferences, decisions, pitfalls. Admission requires: useful across sessions, not greppable in the repo, expressible in one reviewable sentence.
3. **Plain Markdown**, under `~/.niuse/memory/`, files `0600`, optionally its own git repo. No SQLite, no JSONL.
4. **Full injection**, filtered by scope, 2k-token budget. No query-time retrieval, so the system-prompt prefix stays stable and prompt caching survives.
5. **Confirmation before writing**, at most one candidate per turn. The candidate is emitted in-band by the main model; no extra model call.
6. **Soft expiry**: normal within 30 days, marked `unverified` from 31 to 90 days, not injected after 90. Never deleted automatically.
7. **Conflicts are never merged silently**: lexical similarity above threshold offers replace / keep both / discard.
8. **Secret-pattern scan before writing**; a match is refused. Memory is sent to the provider every turn, so the leak surface is much larger than an ordinary file.

## Consequences

**Better:**

- Memory is readable, editable, diffable, and revertible; mistakes are visible and fixable.
- No retrieval dependency, works offline, no embedding service or evaluation set required.
- The stable prompt prefix preserves the prompt-cache behaviour niucode relies on.

**Worse:**

- A hard capacity limit around 200 entries; beyond that, manual merging or pruning is required.
- Recall is "inject everything and let the model judge"; no relevance ranking, and too much memory dilutes attention.
- Candidates depend on the main model volunteering them; missing one is normal.

**Accepted trade-off:** prefer missing a memory to recording a wrong one. A wrong memory enters every subsequent turn and is hard to notice; a missing one costs at most one repeated mistake.

## Alternatives

| Option | Rejected because |
|---|---|
| Vector index over all sessions | No evaluation set, not reviewable, duplicates grep, breaks the stable prefix |
| SQLite storage | No benefit at single-user, 200-entry scale; unreadable to humans and to diffs |
| Automatic distillation | Noisy, and a wrong memory pollutes a turn before anyone can catch it; conflicts with "prefer missing" |
| Project memory in the global store | Duplicates `AGENTS.md` and drifts away from the code |
| Build it into the niucode kernel | Violates niucode's minimal-harness boundary; memory should be a replaceable product layer |
