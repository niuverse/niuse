+++
id = "adr-0007"
title = "ADR-0007: thread.jsonl is the only active line"
kind = "adr"
status = "accepted"
order = 27
summary = "/resume and /fork switch the live session, but thread.jsonl remains the active relationship line across restarts; no pointer file or archive promotion."
+++

## Status

Accepted · 2026-10-22. Clarifies the thread model in
[architecture](../architecture.md) §turn flow and the TUI session commands.

## Context

`~/.niuse/thread.jsonl` is the active relationship line. `threads/*.jsonl` are
archived chapters. The TUI `/resume` and `/fork` commands replace the live
session for the current run, and since niucode's new-session seam they also
notify the embedder hooks. After exit, the next `niuse` resumes
`thread.jsonl`, not the resumed chapter.

Two alternatives were considered:

- **Pointer file.** `~/.niuse/active` names the active path. All readers
  (brief, threads, export, stats) would need to honor it; a stale pointer is a
  new failure mode and a second index.
- **Promotion.** Move the resumed chapter to `thread.jsonl` and archive the
  current line. This rewrites the archive identity, breaks open handles and
  external references, and turns "resume a chapter" into "replace the
  relationship line".

## Decision

1. `thread.jsonl` is the only active line across restarts. There is no pointer
   file and no promotion.
2. `/resume` and `/fork` are **session-scoped views**. They switch the live
   session and update in-process hooks; they do not change which file the next
   launch opens.
3. `/new` and `/reset` archive the current active line and start a new
   `thread.jsonl`; this is the only operation that changes the active line
   across restarts.
4. Commands that read or write "the thread" must use the live session path,
   not a cached `thread.jsonl`, while the TUI is running.
5. If a user wants a chapter to become the main line, they must do it
   explicitly by exporting it or moving files by hand; niuse will not silently
   rewrite the archive.

## Consequences

**Better**

- One active file, no pointer to go stale.
- Archived chapters keep their identity and path.
- `/resume` is predictable: a view for this session, not a permanent switch.

**Worse**

- A resumed chapter is not the next launch's starting point.
- Users who think of `/resume` as "open this conversation" may be surprised;
  the status and help must say "for this session".

## Alternatives

- **Pointer file.** Rejected: second index and stale-pointer failure mode.
- **Promotion by moving the chapter.** Rejected: mutates archive identity and
  breaks open handles.
- **Make `/new` a pointer update.** Rejected: thread.jsonl already has a clear
  physical meaning, and M5-004's seam now preserves it.

## Follow-up

The TUI help/status should call `/resume` session-scoped; no code change is
required for this decision. M5-006 proves the in-session pointer. Cross-restart
promotion is explicitly out of scope.
