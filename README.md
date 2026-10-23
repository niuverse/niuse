# niuse

[![CI](https://github.com/niuverse/niuse/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/niuverse/niuse/actions/workflows/ci.yml)
[![Pages](https://github.com/niuverse/niuse/actions/workflows/pages.yml/badge.svg?branch=main)](https://github.com/niuverse/niuse/actions/workflows/pages.yml)
[![MSRV](https://github.com/niuverse/niuse/actions/workflows/msrv.yml/badge.svg?branch=main)](https://github.com/niuverse/niuse/actions/workflows/msrv.yml)

**niucode is the body. niuse is the identity.**

niuse is a personal engineering companion built on top of [niucode](https://github.com/niuverse/niucode). It keeps your judgements, preferences, and hard-won pitfalls in human-readable Markdown, so the same companion can remember you across projects and sessions, track what it promised, and speak up when you are wrong.

> Status: **runnable MVP**. The seven stores, candidate protocol, soft expiry, bootstrap from history, and an interactive REPL (line editing, history, completion, multi-line input, `/model`, `/cost`, `/export`) are implemented. What is missing is long-term dogfooding in a real terminal.

## Quick start

Requires Rust 1.88+.

```sh
cargo install niuse --version 0.1.12 --locked
niuse                     # enter the REPL
niuse identity init       # eight questions, writes ~/.niuse/identity.md
niuse status              # inspect prompt size and store counts
niuse brief               # overdue promises, review queue, stale memory
niuse bootstrap --limit 5 # mine old niucode sessions for candidates
niuse review              # accept or discard them one by one
```

From a checkout, use `cargo build --release`; the workspace still points at the sibling `../niucode` for development, while the published crate resolves the released `niucode-*` crates.

Model credentials are shared with niucode:

```sh
niucode auth set deepseek
```

## Syncing with niucode

niuse pins niucode in `niucode.rev`; required CI and Pages build against that commit. Move the pin with `./scripts/sync-niucode.sh <rev>`, which runs the contract tests, and see [compat](docs/compat.md) for the matrix. A nightly workflow runs the same tests against niucode `main` so drift surfaces early.

The relationship: **niucode is the kernel and reference UI; niuse is the product and identity.** The front end is moving upstream behind a library target and turn hooks ([ADR-0003](docs/adr/0003-kernel-product-boundary.md)); after that niuse keeps only identity, memory, policy, and product defaults.

## Configuration

`~/.niuse/config.toml` holds defaults; CLI flags always win.

```toml
[defaults]
provider = "deepseek"
model = "deepseek-flash"
sandbox = "workspace-write"
approval = "on-failure"
network = "off"
max_steps = 0
```

`niuse config init` writes a commented template; `config show` prints the file; `config effective` prints the merged values after flags and built-ins. Offline commands never read the file, so a broken config cannot block memory inspection.

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

Injection order is stable-to-volatile: `identity → user → goals → plan → memory → protocol → today → loops → journal`. Only open plan steps are injected. The system prompt is assembled once per session so the provider's prompt cache survives every turn.

**Write thresholds are graded:**

| What | Who decides | Confirmation |
|---|---|---|
| journal, loops | the companion | none |
| memory | it proposes | required |
| identity | you | only you can change it |

## Commands

Global flags: `--preset`, `--model`, `--no-thinking`, `--cwd`, `--sandbox`, `--approval`, `--network`, `--home`, `--thread`, `--max-steps`, `--plan`.

```text
niuse                         interactive REPL (resumes the same thread)
niuse --plan                  start read-only plan mode (read/grep/glob)
niuse exec "<task>"           run one turn; candidates are printed, never written
niuse identity init|show|edit
niuse user show|edit
niuse mem list|search|show|edit|add|forget|confirm|review|prune
niuse recall <query> [--limit N] [--json]  search memory, journal, loops, and goals
niuse loops list|add|done         (list --overdue for late promises)
niuse goals list|add|done         (human-owned outcomes, at most seven open)
niuse journal [--days N] [--search QUERY] [--add TEXT]
niuse threads                   list the active thread and archived chapters
niuse sessions [--json]         list one-shot `exec` session logs
niuse export [--out FILE] [--session PATH]
niuse completions <shell>       print shell completions (bash, zsh, fish, …)
niuse bootstrap [--limit N] [--since-days N] [--dry-run] [--profile] [--profile-only]
niuse review
niuse digest [--thread PATH] [--dry-run] [--limit N]  summarize the thread into the journal
niuse doctor [--json]            check home, identity, credentials, sandbox, pin
niuse config init|show|effective
niuse stats [--days N] [--trend] [--json] local acceptance counters
niuse report [--days N] [--json]        weekly review: turns, cost, stores, dogfood
niuse evidence [--thread PATH] [--json] tools, files, checks, failures, and delegated runs
niuse verify -- <command>          record a check in the thread and exit with it
niuse brief [--json]            what needs attention today
niuse projects [--limit N] [--all] [--json] recent niucode projects
niuse status
```

Install shell completions once:

```sh
niuse completions bash > ~/.local/share/bash-completion/completions/niuse
niuse completions zsh  > "${fpath[1]}/_niuse"
niuse completions fish > ~/.config/fish/completions/niuse.fish
```

`mem list` and `mem search` accept `--kind preference|lesson` and `--tag <tag>`. `mem`, `threads`, `journal`, `loops`, `status`, `doctor`, `stats`, and `config effective` accept `--json` for scripts; `stats` also sums recorded token usage per window and reports cache-hit percent and tokens per turn.

Inside the REPL: `/help`, `/mem`, `/mem recall`, `/mem review`, `/stats`, `/doctor`, `/remember`, `/forget`, `/loops`, `/loop`, `/done`, `/goals`, `/plan`, `/steps`, `/recall`, `/evidence`, `/report`, `/verify`, `/journal search`, `/journal add`, `/identity edit`, `/user`, `/model`, `/cost`, `/threads`, `/sessions`, `/resume`, `/export`, `/new`, `/journal`, `/identity`, `/cd`, `/status`, `/prompt`, `/quit`.

On a terminal the prompt has line editing, persistent history (`~/.niuse/history`, `0600`), Tab completion for slash commands, memory/loop ids, and paths after `/cd` / `/export`; a trailing `\` continues the prompt; piped input keeps the plain reader. Each turn ends with a compact usage line, and tool lines are dimmed unless `NO_COLOR` is set.

Set `NIUSE_MEMORY=off` to disable both memory injection and new candidates, for A/B runs and for recovering trust.

## How it works

One turn:

```text
load the seven stores → compose the prompt (once) → run_with_history
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
  niuse-core/   pure domain: dates, seven stores, prompt, protocol, policy
  niuse-cli/    REPL, commands, provider wiring, session driver, bootstrap
  xtask/        offline dashboard generator
```

`niuse-core` has no IO side effects outside the store layer; its tests need no API key. `niuse-cli` keeps niucode behind a single ~200-line driver.

## Dashboard

```sh
cargo xtask dashboard          # writes docs/index.html
cargo xtask dashboard --check  # fails when the generated board is stale
```

Four views: Architecture (default), Work, Docs, Decisions. Sources are `docs/dashboard/project.json`, `docs/board/*.md`, and `docs/**/*.md`; see [docs/README.md](docs/README.md) for the maintenance ritual.

`.github/workflows/pages.yml` deploys the generated board to GitHub Pages. A private repository needs GitHub Pro/Team/Enterprise; on the Free plan the workflow skips with a notice until Pages is available.

## Web

Serve a local, loopback-only browser surface with the same identity, user,
memory, goals, and plan prompt as the REPL:

```sh
cargo run --features web --bin niuse -- web --port 8788
```

The command prints a tokenized URL; open that exact URL. Bare loopback URLs are
rejected so other local pages cannot drive the agent. The `web` feature is
opt-in for the same reason as `tui`: the default binary does not link niucode.
It reuses niucode's local SSE server, browser approvals, Markdown, and MathML. The session is `~/.niuse/thread.jsonl`. The sidebar shows
identity, user, goals, plan, memory, open loops, and the 7-day report from the stores. Chat is
workspace-write with explicit browser approval; there is no remote mode.

## Experimental TUI

The default REPL stays on rustyline. An opt-in build reuses niucode's TUI with niuse hooks:

```sh
cargo run --features tui -- tui
# or: cargo install niuse --features tui
```

Memory candidates become accept/edit/discard prompts after the turn; conflicts offer replace/keep both. Journal and loops are written automatically. `NIUSE_MEMORY=off` disables injection and candidates in the TUI too. `/brief`, `/goals`, `/plan`, `/recall`, `/report`, `/mem`, `/mem show <id>`, `/loops`, `/journal`, `/status`, `/cost`, `/threads`, `/sessions`, `/export`, `/prompt`, `/identity`, and `/user` are handled by the embedder command hook and appear in `/help` and Tab completion. The default binary does not link the TUI. `--thread` also selects the TUI's active session. `/new` is an alias of `/clear`: it archives the current thread and starts a fresh `thread.jsonl`; `/resume` and `/fork` switch only the live session (ADR-0007).

## MCP

All paths mount read-only MCP servers by default. The quick start installs both
bundled packs and writes `~/.niucode/mcp.toml`:

```sh
niuse mcp init --notes ~/notes --calendar ~/calendar.ics
niuse tui
```

`mcp init` copies the bundled servers to `~/.niuse/packs/`, resolves an
absolute `python3`, and writes only `read_only = true` entries. It merges by
server name, preserves existing text and comments, and refuses to touch an
invalid manifest; `--force` replaces the file. You can also write the manifest
by hand; tools appear as `mcp__<server>__<tool>`, and the TUI accepts
`--mcp-config`.

A broken manifest or a server that fails to start prints a warning and disables
MCP for that run; it does not take down the session. A read-only notes pack is
included: [packs/notes](packs/notes/README.md) and
[packs/calendar](packs/calendar/README.md). See niucode's
[ADR-0033](https://github.com/niuverse/niucode/blob/main/docs/adr/0033-mcp-client.md).

A manifest may opt into writes with `read_only = false` and an explicit
`write_tools` list. `niuse exec` and `niuse tui` mount it only with
`--allow-mcp-write` and an interactive approval mode; every mutating call shows
the tool and bounded arguments. `--approval never` and non-interactive runs
mount read-only servers only. The `mcp init` generator still writes read-only
entries; a write manifest is hand-written and audited against
[ADR-0034](https://github.com/niuverse/niucode/blob/main/docs/adr/0034-mutating-mcp.md).

## Privacy

Everything niuse stores is local Markdown under `~/.niuse`: directories `0700`, files `0600`. Writers create files private from the first byte; `$EDITOR` scratch files and exports are `0600` too. Memory, journal, loops, identity, and user text are scanned for common secret patterns before writing; a match is refused. `NIUSE_MEMORY=off` disables injection entirely for an A/B run or trust recovery.

History (`~/.niuse/history`) contains the prompts you typed, and thread/session logs contain transcripts; both are covered by the same permissions. Nothing leaves the machine except the provider calls you configured.

## Development

```sh
./scripts/check.sh   # fmt, clippy, docs, tests, package, dashboard
./scripts/eval.sh    # offline personal-agent eval + notes MCP pack
./scripts/dogfood-trend.sh          # compare the last two weeks of real use
./scripts/dogfood-trend.sh --strict # warnings also fail
```

Repository rules and commands for future sessions live in [AGENTS.md](AGENTS.md).

## Not building

A daemon, a web UI, WeChat / Feishu bridges, scheduled push notifications, vector search, full session indexing, automatic repo edits, or multi-user support. The TUI is opt-in; the working-companion scope is in [roadmap](docs/roadmap.md).

## Known limits

- One active line at a time: `thread.jsonl`. `/new` and `/reset` archive it and start a fresh one; `/resume` and `/fork` are session-scoped views, not a change to the next launch's active line ([ADR-0007](docs/adr/0007-active-thread.md)).
- Memory is capped around 200 entries; overflow is handled by budget trimming and manual review.
- Journal, memory, and stats day labels follow the machine's local timezone; event timestamps stay UTC.
- `bootstrap` sends old user/assistant text to the provider (tool output is dropped).
- Similarity uses character-bigram Jaccard; the thresholds are heuristics and will be tuned after two weeks.

## Docs

- [Status](docs/status.md): what works and what waits on a decision
- [Implementation plan](docs/plan.md): decisions, stores, turn lifecycle, milestones, acceptance.
- [Architecture](docs/architecture.md): boundaries, data flow, write thresholds.
- [ADR-0001: memory layer](docs/adr/0001-memory-layer.md)
- [ADR-0002: identity layer](docs/adr/0002-identity-layer.md)
- [ADR-0003: kernel and product boundary](docs/adr/0003-kernel-product-boundary.md)
- [niucode compatibility](docs/compat.md)
- [Dogfood log](docs/dogfood.md): the ten-day acceptance ritual
- [Distribution notes](docs/distribution.md): install paths, crates.io blockers, license decision
- [Agent guide](AGENTS.md): repo rules for future sessions

## License

MIT OR Apache-2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
