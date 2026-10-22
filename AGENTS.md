# AGENTS.md

This repository is `niuse`, a personal engineering companion built on top of
`niucode`. Keep it small, reviewable, and honest about what it remembers.

## Read first

1. `docs/status.md` — current snapshot and the decisions waiting on a human.
2. `docs/roadmap.md` — the long-running goal, phases, and current sprint.
3. `docs/architecture.md` — boundaries, stores, turn flow, non-goals.
4. `docs/plan.md` — decisions, milestones, acceptance.
5. `docs/compat.md` — pinned niucode revision and the weekly bump ritual.
6. `docs/dashboard/project.json` and `docs/board/` — current work and decisions.

## Commands

```sh
./scripts/check.sh              # fmt, clippy, docs, tests, package, dashboard
cargo xtask dashboard           # regenerate docs/index.html
cargo xtask dashboard --check   # fail when the generated board is stale
./scripts/sync-niucode.sh <rev> # bump the pin, run tests, remind about compat
./scripts/check-niucode-drift.sh
cargo +1.88.0 check --workspace --all-targets   # MSRV; weekly workflow
./scripts/dogfood.sh "note"     # record one real dogfood day
cargo test -p niuse --features tui   # opt-in TUI path
```

`cargo xtask` is a workspace alias defined in `.cargo/config.toml`.

## Hard rules

- **Two repositories.** `niucode` is the kernel and reference UI; `niuse` is
  the product and identity. Never fork niucode files into this repo.
- **Pin what you consume.** `niucode.rev` is the revision required CI builds
  against. Bump it through the script, not by editing the number by hand.
- **Markdown is the only storage.** `identity.md`, `user.md`, `memory/`,
  `journal/`, `loops.md`, `goals.md` live under `~/.niuse`; no database, no
  second index.
- **Confirmation is the write barrier for memory.** Journal and loops are
  automatic; memory is proposed and confirmed; identity is human-written.
- **Never auto-edit a user's repository.** Project knowledge stays in that
  repo's `AGENTS.md`.
- **Secrets are refused at the write.** Identity and user text is injected on
  every turn, so it gets the same scan as memory.
- **Create private, do not repair.** Store files, thread/session logs, exports,
  metrics, and `$EDITOR` scratch files are created at `0600` with a unique
  name. A chmod is only for pre-existing loose files. Directories are `0700`.
- **Docs and README stay in English.** Runtime strings and the persona prompt
  may be Chinese; that is product behaviour, not documentation.
- **TUI is opt-in.** `niuse tui` lives behind the `tui` feature; the default binary does not link niucode's TUI. `render.rs` stays until ADR-0004's ten real TUI dogfood days and parity gate.
- **Commit and push after a verified increment.** Do not leave a green change
  uncommitted; do not push a red one.

## Layout

```text
crates/niuse-core/   domain: dates, six stores, prompt, protocol, policy
crates/niuse-cli/    REPL, subcommands, driver, bootstrap, export, tui_hook
crates/xtask/        offline dashboard generator
docs/                plan, architecture, ADRs, board cards, generated board
scripts/             check, niucode sync, drift check
```

`commands/` is split by domain: memory, loops, journal, profile, session,
doctor, editor, with `mod.rs` keeping status/stats/metrics and shared helpers.

Runtime state: `thread.jsonl` (active line), `threads/` (archived chapters),
`sessions/` (one-shot `exec` logs), `backups/` (prune snapshots), `history`
(prompt history), `metrics.jsonl` (counters). `niuse threads` / `sessions`
list them; `/threads`, `/sessions`, and `/resume <n>` do it inside the REPL.

## Testing expectations

- All tests are offline; no API key is needed. Use `ScriptedProvider` for
  turn-level tests.
- The contract test in `crates/niuse-cli/tests/niucode_contract.rs` is the
  canary for niucode changes: system prompt, history projection, tools,
  per-turn cwd, and approval denial.
- Prefer a new focused test over a broad fixture. Edge cases in protocol
  parsing and secret scanning deserve a test each.
- The `tui` feature has its own tests; `scripts/check.sh` and the MSRV job
  both cover it. Keep `filter_text`/`after_turn`/`on_answer` parity with the
  REPL in mind when changing the TUI adapter.
- Session logs must stay forward-compatible: unknown event types warn while
  readable lines survive; unknown fields on known events are ignored. niucode
  pins both behaviors; niuse relies on them for the pinned revision.
