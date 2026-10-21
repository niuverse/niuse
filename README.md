# niuse

[![CI](https://github.com/niuverse/niuse/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/niuverse/niuse/actions/workflows/ci.yml)
[![Pages](https://github.com/niuverse/niuse/actions/workflows/pages.yml/badge.svg?branch=main)](https://github.com/niuverse/niuse/actions/workflows/pages.yml)

**niucode is the body. niuse is the identity.**

niuse is a personal engineering companion built on top of [niucode](https://github.com/niuverse/niucode). It keeps your judgements, preferences, and hard-won pitfalls in human-readable Markdown, so the same companion can remember you across projects and sessions, track what it promised, and speak up when you are wrong.

> Status: **runnable MVP**. The five stores, candidate protocol, soft expiry, bootstrap from history, and the REPL are implemented. What is missing is long-term dogfooding in a real terminal.

## Quick start

Requires Rust 1.85+ and a local checkout of `../niuverse/niucode` (the current build uses path dependencies; a pinned git dependency is planned).

```sh
cargo build --release

./target/release/niuse                     # enter the REPL
./target/release/niuse identity init       # eight questions, writes ~/.niuse/identity.md
./target/release/niuse status              # inspect prompt size and store counts
./target/release/niuse bootstrap --limit 5 # mine old niucode sessions for candidates
./target/release/niuse review              # accept or discard them one by one
```

Model credentials are shared with niucode:

```sh
niucode auth set deepseek
```

## Syncing with niucode

niuse pins niucode in `niucode.rev`; required CI and Pages build against that commit. Move the pin with `./scripts/sync-niucode.sh <rev>`, which runs the contract tests, and see [compat](docs/compat.md) for the matrix. A nightly workflow runs the same tests against niucode `main` so drift surfaces early.

The relationship: **niucode is the kernel and reference UI; niuse is the product and identity.** The front end is moving upstream behind a library target and turn hooks ([ADR-0003](docs/adr/0003-kernel-product-boundary.md)); after that niuse keeps only identity, memory, policy, and product defaults.

## What it remembers

```text
~/.niuse/
  identity.md      who it is (you write it; it may only propose changes)
  user.md          who you are (bootstrapped from history, reviewed by you)
  memory/          facts: preferences, decisions, pitfalls (confirmed, expires)
  journal/         experience: written by the companion after sessions (no confirmation)
  loops.md         promises it made, with optional due dates
  thread.jsonl     the relationship thread (a resumable niucode session)
  inbox.md         review queue produced by bootstrap
  metrics.jsonl    local acceptance counters (no memory text)
```

Injection order is stable-to-volatile: `identity → user → memory → protocol → today → loops → journal`. The system prompt is assembled once per session so the provider's prompt cache survives every turn.

**Write thresholds are graded:**

| What | Who decides | Confirmation |
|---|---|---|
| journal, loops | the companion | none |
| memory | it proposes | required |
| identity | you | only you can change it |

## Commands

Global flags: `--preset`, `--model`, `--no-thinking`, `--cwd`, `--sandbox`, `--approval`, `--network`, `--home`, `--max-steps`.

```text
niuse                         interactive REPL (resumes the same thread)
niuse exec "<task>"           run one turn; candidates are printed, never written
niuse identity init|show
niuse mem list|search|add|forget|confirm|review
niuse loops list|done
niuse journal [--days N]
niuse bootstrap [--limit N] [--since-days N]
niuse review
niuse doctor                    check home, identity, credentials, sandbox, pin
niuse stats [--days N]          local acceptance counters
niuse status
```

Inside the REPL: `/help`, `/mem`, `/mem recall`, `/mem review`, `/stats`, `/doctor`, `/remember`, `/forget`, `/loops`, `/done`, `/journal`, `/identity`, `/cd`, `/status`, `/prompt`, `/quit`.

Set `NIUSE_MEMORY=off` to disable both memory injection and new candidates, for A/B runs and for recovering trust.

## How it works

One turn:

```text
load the five stores → compose the prompt (once) → run_with_history
                     → stream, with a tail buffer that strips protocol blocks
                     → journal / loop blocks are written automatically
                     → memory candidates: y / n / e / s
```

The model may end an answer with three block types. niuse parses them and removes them from the visible text:

```text
<<niuse:memory kind=preference tags=rust,testing>>
one self-contained conclusion
<</niuse:memory>>

<<niuse:loop due=2026-10-22>>
something it promised to do
<</niuse:loop>>

<<niuse:journal>>
- a decision or turning point that differs from last time
<</niuse:journal>>
```

No second model call is involved. Missing a memory is acceptable; recording a wrong one is not.

## Layout

```text
crates/
  niuse-core/   pure domain: dates, five stores, prompt, protocol, policy
  niuse-cli/    REPL, commands, provider wiring, session driver, bootstrap
  xtask/        offline dashboard generator
```

`niuse-core` has no IO side effects outside the store layer; its 35 tests need no API key. `niuse-cli` keeps niucode behind a single ~200-line driver.

## Dashboard

```sh
cargo xtask dashboard          # writes docs/index.html
cargo xtask dashboard --check  # fails when the generated board is stale
```

Four views: Architecture (default), Work, Docs, Decisions. Sources are `docs/dashboard/project.json`, `docs/board/*.md`, and `docs/**/*.md`; see [docs/README.md](docs/README.md) for the maintenance ritual.

`.github/workflows/pages.yml` deploys the generated board to GitHub Pages. A private repository needs GitHub Pro/Team/Enterprise; on the Free plan the workflow skips with a notice until Pages is available.

## Not building

A daemon, a web UI, WeChat / Feishu bridges, scheduled push notifications, vector search, full session indexing, automatic repo edits, multi-user support, a TUI (for now), or a general agent framework.

## Known limits

- One thread: a single `thread.jsonl`, no `/new`.
- Approval shows a summary and +/- line counts, not a structured diff yet.
- Memory is capped around 200 entries; overflow is handled by budget trimming and manual review.
- Dates are computed in UTC, so near midnight the journal may be one day off.
- `bootstrap` sends old user/assistant text to the provider (tool output is dropped).
- Similarity uses character-bigram Jaccard; the thresholds are heuristics and will be tuned after two weeks.

## Docs

- [Implementation plan](docs/plan.md): decisions, stores, turn lifecycle, milestones, acceptance.
- [Architecture](docs/architecture.md): boundaries, data flow, write thresholds.
- [ADR-0001: memory layer](docs/adr/0001-memory-layer.md)
- [ADR-0002: identity layer](docs/adr/0002-identity-layer.md)
- [ADR-0003: kernel and product boundary](docs/adr/0003-kernel-product-boundary.md)
- [niucode compatibility](docs/compat.md)
