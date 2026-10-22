# Changelog

## Unreleased

- TUI embedder commands register in `/help` and Tab completion; `/mem`, `/mem show`, `/loops`, `/journal`, and `/status` are wired through `CompanionHooks`.

## 0.1.4 — 2026-10-22

- Optional `tui` feature: `niuse tui` runs niucode's TUI with `CompanionHooks`; protocol blocks are stripped, journal/loops are automatic, memory uses accept/edit/discard prompts, conflicts offer replace/keep both, and read-only `/mem`, `/loops`, `/journal`, `/status` are handled through the command hook.

## 0.1.3 — 2026-10-22

- When `identity.md` is missing, the prompt now gives an exact one-line answer and forbids tool calls, so "who are you" no longer turns into a glob/bash detour.

## 0.1.2 — 2026-10-22

- Missing `identity.md` now yields an explicit "not named yet" section instead of letting the model invent a name.

## 0.1.1 — 2026-10-22

- `identity.md` is now the only name source; the persona no longer hardcodes "you are niuse", and identity precedence is explicit.

## 0.1.0 — 2026-10-22

First runnable niuse.

### Companion
- Five Markdown stores: `identity.md`, `user.md`, `memory/`, `journal/`, `loops.md`; directories `0700`, files `0600`.
- In-band protocol blocks for memory, loops, and journal; stripped from the stream and from exports.
- Memory soft expiry, conflict replace/keep, secret refusal, confirmation-gated writes, prune with backup.
- Journal, loop promises with due dates and an `--overdue` view, and identity/user editing.

### Interactive
- Streaming REPL with line editing, persistent history, Tab completion for commands, ids, and paths.
- Backslash continuation, `/new` chapters, `/resume`, `/threads`, `/sessions`, `/export`, `/cost`, `/model`.
- Approval diffs with a full-diff option; partial denials return an approval preview.
- Slash-command errors report and keep the session alive.

### Learning
- Bootstrap from niucode session logs: memory candidates, user profile draft, dry-run, review queue.
- JSON views for memory, journal, threads, sessions, loops, status, doctor, stats, config, identity, and user.
- Acceptance and token metrics with local-calendar daily trends.

### Ops
- `doctor`, MSRV 1.88, pinned niucode revision, contract tests, shell completions.
- Local calendar dates for journal, memory, and stats; event timestamps stay UTC.
- Security invariant: every writer creates its file `0600`; chmod only repairs old files.
