+++
id = "status"
title = "Status"
kind = "doc"
status = "living"
order = 1
summary = "Entry point after the autonomous iteration: what works, what is pinned, and what waits for a human decision."
+++

## Snapshot · 2026-10-22

niuse is runnable end to end: five Markdown stores, an interactive REPL with line editing, history, completion, multi-line input, confirmation and conflict handling, the in-band protocol, bootstrap from history (memory candidates and a user profile), JSON views for every store, one-shot session listing, colored approval diffs, `doctor`, and acceptance metrics. The declared MSRV is 1.88; the full test suite passes under it, and a weekly workflow checks it.

Both repositories are green. The board at <https://niuverse.github.io/niuse/> is deployed from a single generated file; source docs are not published. niuse builds against the niucode revision in `niucode.rev`.

## Verify

```sh
./scripts/check.sh              # fmt, clippy, docs, tests, package, dashboard
./scripts/check-niucode-drift.sh
cargo xtask dashboard --check
cargo +1.88.0 check --workspace --all-targets   # MSRV, also run weekly
```

## Works

- `niuse` REPL: streaming, slash commands, line editing and history, Tab completion for commands/ids/paths, backslash continuation, `/new` chapters, `/cd`, `/model`, `/cost`, `/threads`, `/sessions`, `/resume`, `/export`, candidate confirm/edit/skip, conflict replace/keep.
- Memory: search including expired entries, kind filter, kind counts, show/edit, review with metrics, prune with backup and freed-byte report, duplicate guard, secret scan, soft expiry.
- Dates: journal, memory, and stats day labels follow the local calendar; event timestamps stay UTC.
- Journal and loops: model-written blocks plus manual entry and manual promises.
- Bootstrap: dry-run preview, candidate merge into the inbox, optional `user.md` draft.
- Sessions: derived thread titles, `threads` and `sessions` listings, Markdown export with a title, JSON output.
- Machine output: JSON for memory (list/search/show), journal, threads, sessions, loops, status, doctor, stats, config effective, identity, and user; `stats` totals and trends include token usage.
- Operations: pinned niucode revision, contract tests, nightly main check, doctor, acceptance metrics.
- Distribution: the seven niucode crates are published at 0.1.0; `cargo install niucode --version 0.1.0 --locked` was verified from a clean root. niuse itself is not on crates.io yet.

## Waiting on you

1. **Dogfood** — ten working days recorded in [dogfood](dogfood.md). `./scripts/dogfood.sh "note"` writes the day's row from `stats --days 1 --json`; only real use can close M2.

## In progress

- **ADR-0032 step 2** — the library target and `TurnHooks` are exported; the exec path now invokes prompt augmentation, event observation, and turn-end (`run_exec_with_hooks`). The TUI path and renderer migration remain. Discussion: [niuverse/niucode#1](https://github.com/niuverse/niucode/issues/1).

## Not built

Daemon, web UI, messaging bridges, scheduled push, vector search, full session indexing, automatic repo edits, multi-user support, a TUI (until the upstream seam lands).
