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
- Journal and loops: model-written blocks plus manual entry and manual promises.
- Bootstrap: dry-run preview, candidate merge into the inbox, optional `user.md` draft.
- Sessions: derived thread titles, `threads` and `sessions` listings, Markdown export with a title, JSON output.
- Machine output: JSON for memory, journal, threads, loops, status, doctor, stats; `stats` totals and trends include token usage.
- Operations: pinned niucode revision, contract tests, nightly main check, doctor, acceptance metrics.

## Waiting on you

1. **ADR-0032** — accept or reject the reusable TUI library and turn hooks. If accepted, the next step is the library target plus empty hooks, then moving niuse's renderer upstream. Discussion: [niuverse/niucode#1](https://github.com/niuverse/niucode/issues/1).
2. **License and dependency strategy** — publishing needs a license choice and a decision between publishing niucode crates first or staying on `cargo install --git`. Details: [distribution](distribution.md).
3. **Dogfood** — ten working days recorded in [dogfood](dogfood.md). The tooling is ready; only real use can close M2.

## Not built

Daemon, web UI, messaging bridges, scheduled push, vector search, full session indexing, automatic repo edits, multi-user support, a TUI (until the upstream seam lands).
