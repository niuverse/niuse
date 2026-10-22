+++
id = "status"
title = "Status"
kind = "doc"
status = "living"
order = 1
summary = "Entry point after the autonomous iteration: what works, what is pinned, and what waits for a human decision."
+++

## Snapshot · 2026-10-22

niuse is runnable end to end: six Markdown stores, an interactive REPL with line editing, history, completion, multi-line input, confirmation and conflict handling, the in-band protocol, a read-only startup brief, bootstrap from history (memory candidates and a user profile), JSON views for every store, one-shot session listing, colored approval diffs, `doctor`, and acceptance metrics. The declared MSRV is 1.88; the full test suite and the optional `tui` feature pass under it, and a weekly workflow checks both.

Both repositories are green. The board at <https://niuverse.github.io/niuse/> is deployed from a single generated file; source docs are not published. niuse builds against the niucode revision in `niucode.rev`. The long-running direction is in [roadmap](roadmap.md): harness-native companion, compounding context, long-horizon work, connected tools, and proof.

## Verify

```sh
./scripts/check.sh              # fmt, clippy, docs, tests, package, dashboard
./scripts/check-niucode-drift.sh
cargo xtask dashboard --check
cargo +1.88.0 check --workspace --all-targets   # MSRV, also run weekly
```

## Works

- `niuse` REPL: streaming, slash commands, line editing and history, Tab completion for commands/ids/paths, backslash continuation, `/new` chapters, `/cd`, `/model`, `/cost`, `/threads`, `/sessions`, `/resume`, `/export`, `/brief`, `/plan`, candidate confirm/edit/skip, conflict replace/keep.
- Brief: `niuse brief [--json]`, `/brief` in the REPL and TUI, and a compact actionable block after the banner; no provider call and `NIUSE_BRIEF=off` restores the quiet start.
- Projects: `niuse projects [--limit N] [--all] [--json]` reads niucode's recorded workspaces, sorts by recent activity, hides missing and temporary test checkouts by default, and shows the current project in `status`.
- Digest: `niuse digest [--thread PATH] [--dry-run] [--limit N]` compresses recent user/assistant prose into at most five journal bullets with one tool-free model call; tool output never enters the prompt, secrets refuse the whole write, and the path stays opt-in until ten real digests are reviewed.
- `niuse tui` (optional feature): niucode's TUI with `CompanionHooks`; protocol stripping, journal/loops, memory accept/edit/replace/keep, read-only commands, `/brief`, `/export`, `y`/`n`/`s` approval keys, and `/cost` read from the thread event log.
- Memory: search including expired entries, kind filter, kind counts, show/edit, review with metrics, prune with backup and freed-byte report, duplicate guard, secret scan, soft expiry.
- Recall: `niuse recall <query> [--json]` and `/recall` search memory (including expired), journal, and loops together; the agent can also call the read-only `memory_search` tool during a turn, and the protocol tells it to request `/recall` when no tool is available.
- Dates: journal, memory, and stats day labels follow the local calendar; event timestamps stay UTC.
- Journal and loops: model-written blocks plus manual entry and manual promises.
- Goals: `goals.md` holds human-owned outcomes with `g-` ids; `niuse goals list|add|done` and `/goals` edit them, at most seven open, secret-scanned, and injected after user and before memory.
- Bootstrap: dry-run preview, candidate merge into the inbox, optional `user.md` draft.
- Sessions: derived thread titles, `threads` and `sessions` listings, Markdown export with a title, JSON output.
- Machine output: JSON for memory (list/search/show), journal, threads, sessions, loops, status, doctor, stats, config effective, identity, and user; `stats` totals and trends include token usage.
- Operations: pinned niucode revision, contract tests, nightly main check, doctor, acceptance metrics.
- Distribution: seven niucode crates and both niuse crates are published. niucode 0.1.3 adds `filter_text`, the embedder command hook, and post-turn prompts on top of 0.1.2's full TUI hook surface; `cargo install niucode --version 0.1.3` and `cargo install niuse --version 0.1.6` were verified from clean roots, and `cargo package -p niuse` succeeds against the registry.

## Waiting on you

1. **Dogfood** — ten working days recorded in [dogfood](dogfood.md). `./scripts/dogfood.sh "note"` writes the day's row from `stats --days 1 --json`; only real use can close M2.
2. **TUI dogfood** — ten real days on `niuse tui` for M5-002; this is also the ADR-0004 gate for retiring the REPL renderer.
3. **Digest review** — review ten real `niuse digest` runs before treating it as automatic; the card stays in `review` until then.

## In progress

- **M5 · Harness-native companion** — M5-001 is done: configurable `y`/`n`/`s` approval keys, an offline PTY deny/allow test, product commands in TUI help/completion, and `/cost` computed from the thread event log. M5-002 starts the ten-day TUI dogfood gate.
- **M6 · Compounding context** — startup brief (M6-001), project registry (M6-002), session digest (M6-003, in review), unified recall (M6-004), and agent-invoked `memory_search` (M6-005) landed. The remaining M6 item is real digest review.
- **M7 · Long-horizon work** — the human-owned goal store (M7-001) and read-only REPL/`exec` plan mode (M7-002) landed; execute → verify with recorded evidence and delegated work are next.
- **M2 dogfood** continues in parallel; it is not a blocker for product work, but it still decides whether the memory layer pays for itself.

## Not built

Daemon, web UI, messaging bridges, scheduled push, vector search, full session indexing, automatic repo edits, and multi-user support remain out until a new ADR. The TUI is opt-in, not the default; see [roadmap](roadmap.md) for the order.
