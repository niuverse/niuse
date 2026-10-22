# Changelog

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
