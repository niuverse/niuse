# AGENTS.md

This repository is `niuse`, a personal engineering companion built on top of
`niucode`. Keep it small, reviewable, and honest about what it remembers.

## Read first

1. `docs/architecture.md` — boundaries, stores, turn flow, non-goals.
2. `docs/plan.md` — decisions, milestones, acceptance.
3. `docs/compat.md` — pinned niucode revision and the weekly bump ritual.
4. `docs/dashboard/project.json` and `docs/board/` — current work and decisions.

## Commands

```sh
./scripts/check.sh              # fmt, clippy, tests, dashboard freshness
cargo xtask dashboard           # regenerate docs/index.html
cargo xtask dashboard --check   # fail when the generated board is stale
./scripts/sync-niucode.sh <rev> # bump the pin, run tests, remind about compat
./scripts/check-niucode-drift.sh
```

`cargo xtask` is a workspace alias defined in `.cargo/config.toml`.

## Hard rules

- **Two repositories.** `niucode` is the kernel and reference UI; `niuse` is
  the product and identity. Never fork niucode files into this repo.
- **Pin what you consume.** `niucode.rev` is the revision required CI builds
  against. Bump it through the script, not by editing the number by hand.
- **Markdown is the only storage.** `identity.md`, `user.md`, `memory/`,
  `journal/`, `loops.md` live under `~/.niuse`; no database, no second index.
- **Confirmation is the write barrier for memory.** Journal and loops are
  automatic; memory is proposed and confirmed; identity is human-written.
- **Never auto-edit a user's repository.** Project knowledge stays in that
  repo's `AGENTS.md`.
- **Secrets are refused at the write.** Identity and user text is injected on
  every turn, so it gets the same scan as memory.
- **Docs and README stay in English.** Runtime strings and the persona prompt
  may be Chinese; that is product behaviour, not documentation.
- **Commit and push after a verified increment.** Do not leave a green change
  uncommitted; do not push a red one.

## Layout

```text
crates/niuse-core/   domain: dates, five stores, prompt, protocol, policy
crates/niuse-cli/    REPL, subcommands, driver, bootstrap, export
crates/xtask/        offline dashboard generator
docs/                plan, architecture, ADRs, board cards, generated board
scripts/             check, niucode sync, drift check
```

`commands/` is split by domain: memory, loops, journal, profile, session,
doctor, editor, with `mod.rs` keeping status/stats/metrics and shared helpers.

## Testing expectations

- All tests are offline; no API key is needed. Use `ScriptedProvider` for
  turn-level tests.
- The contract test in `crates/niuse-cli/tests/niucode_contract.rs` is the
  canary for niucode changes: system prompt, history projection, tools,
  per-turn cwd, and approval denial.
- Prefer a new focused test over a broad fixture. Edge cases in protocol
  parsing and secret scanning deserve a test each.
