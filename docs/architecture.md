+++
id = "architecture"
title = "Architecture"
kind = "doc"
status = "living"
order = 5
summary = "niuse boundaries, the seven stores, turn data flow, write thresholds, and explicit non-goals."
+++

## In one sentence

niuse is the identity and memory layer above niucode: it decides who the companion is, what it remembers, and what it promised. Whether a task can be done at all stays with niucode's loop, tools, and sandbox.

## Dependency direction

```text
crates/niuse-cli     REPL / commands / renderer / driver / bootstrap
    │
crates/niuse-core    date / model / store / prompt / candidate / policy
    │
niucode-agent        run_with_history / AgentConfig / project_history
niucode-core         SessionLog / Event
niucode-provider     Provider / ChatMessage
niucode-auth         AuthStore
niucode-tools        SandboxPolicy / ApprovalMode
```

Only `driver.rs` and `memory_tool.rs` touch `niucode-agent` directly. Everything else is either pure domain logic or only knows `niuse-core` types. When niucode upgrades, the required changes are confined to those adapters.

niuse does not fork niucode and does not add a memory hook to its loop. Memory is a product layer; the kernel stays minimal.

niucode exposes a read-only `EmbedderTool` seam (ADR-0032): niuse registers `memory_search` through it in the REPL, `exec`, and TUI. The tool reuses unified recall, is advertised as read-only, and cannot write, delete, or confirm memory.

niucode now exposes a library target and the generic `TurnHooks` trait (ADR-0032). Both `run_exec_with_hooks` and `run_tui_with_hooks` invoke prompt, event, and turn-end hooks; the renderer migration remains. niuse keeps its REPL and renderer deliberately rather than accidentally until that migration lands.

Terminal input uses `rustyline` only when stdin is a TTY; pipes keep the plain reader. History is stored at `~/.niuse/history` with mode `0600`. A line ending in `\` continues the prompt and is joined into one user message.

## Modules

| Module | Layer | Responsibility |
|---|---|---|
| `cli` | Interface | flags, subcommands, offline commands, wiring |
| `repl` | Interface | conversation, slash commands, candidate confirmation, rendering |
| `prompt` | Engine | stable-prefix system prompt and budgets |
| `brief` | Engine | read-only "what needs attention" view over the stores |
| `protocol` | Engine | in-band block parsing and streaming stripping |
| `policy` | Engine | soft expiry, conflicts, secret scanning |
| `driver` | Adapters | niucode boundary, sessions, per-turn cwd |
| `projects` | Adapters | read-only view of niucode's project registry |
| `bootstrap` | Adapters | old-session mining and the inbox |
| `stores` | State | seven-store load / save and permissions |
| `dashboard` | Engineering | offline board generation |

## Active thread

`thread.jsonl` is the only active line across restarts; `threads/*.jsonl` are
archived chapters. TUI `/resume` and `/fork` switch the live session for that
run and notify embedder hooks, but they do not change the file the next launch
opens. `/new` and `/reset` archive the current line and start a new
`thread.jsonl`. See [ADR-0007](adr/0007-active-thread.md).

## Turn data flow

```text
startup
  ├─ read identity / user / memory / loops / journal
  ├─ compose() assembles the system prompt (once per session)
  ├─ brief: overdue loops, inbox, stale/expired memory (read-only)
  └─ project_history(thread.jsonl) restores history

user input
  ├─ run_with_history(config, prompt, history)
  ├─ AgentEvent → Renderer (TextDelta goes through StreamFilter)
  ├─ trailing buffer strips <<niuse:*>> blocks
  ├─ journal / loop: written automatically
  ├─ memory: print candidate → y / n / e / s
  └─ re-project history, next turn
```

There is no second copy of the conversation. `niuse export` renders the event log as Markdown without changing it. `history` is projected from the event log by niucode's `project_history`, so compaction, replay, and fork behave exactly as they do in niucode.

## The seven stores and injection

```text
~/.niuse/
  identity.md      who it is (human-written; it may only propose)
  user.md          who you are (bootstrapped with --profile, human-reviewed)
  memory/          facts: preferences, decisions, pitfalls (confirmed, expiring)
  journal/         experience: one file per month, merged per day (automatic)
  loops.md         promises (automatic, optional due date)
  goals.md         human-owned outcomes (injected, at most seven open)
  plan.md          human-owned steps (open ones injected; closed only with verification)
  thread.jsonl     the relationship thread
  threads/         archived chapters from `/new`
  sessions/        one-shot `exec` logs (`niuse sessions`)
  inbox.md         bootstrap review queue
  history          TTY prompt history (0600)
  backups/         prune snapshots (0700)
  metrics.jsonl    local acceptance counters
  config.toml      defaults and overrides
```

Defaults come from CLI flags, then `~/.niuse/config.toml`, then built-ins; only commands that build a provider read the file.

Injection order is fixed, stable before volatile:

| Section | Budget | Change rate |
|---|---|---|
| persona | constant | never |
| identity | ≤ 600 tok | very low |
| user | ≤ 600 tok | low |
| goals | ≤ 300 tok | low |
| memory | ≤ 2 000 tok | low |
| protocol | constant | never |
| today | one line | daily |
| loops | ≤ 300 tok | medium |
| journal | ≤ 600 tok | high |

The first five sections form the stable cache prefix; only content after `today` can vary per session. The system prompt is assembled once and is not rebuilt mid-session. `identity.md` is the only source of the companion's name and voice; the persona describes working style and the product name is not a self-name. When identity is missing, the prompt says so explicitly instead of inventing a name.

## Write thresholds and protocol

| What | Who decides | Confirmation |
|---|---|---|
| journal, loops | the companion | none |
| memory | it proposes | required |
| identity | the human | only the human can change it |

The model may end an answer with three block types:

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

At most one `memory` block per turn; after a rejection, the same tags are not proposed again in that session. `journal` records changes, not activity logs. Expired memory leaves the prompt but stays searchable through `niuse mem search` and `/mem recall`.

## Safety and privacy

- `~/.niuse` directories are `0700`, files are `0600`. Writers create the file at `0600` from the first byte (unique temp name, then rename); a chmod only repairs pre-existing files. `$EDITOR` scratch files, exports, and thread logs follow the same rule.
- `mem prune --apply` copies memory files to `~/.niuse/backups/` before deleting.
- Secret patterns are scanned before every write; a match is refused, and the CLI requires `--force` to override where an override exists.
- `identity.md` and `user.md` edits, and the bootstrap profile draft, are refused on a secret match; these files are injected every turn.
- Manual `mem add` runs the same conflict rule as candidates; `--force` overrides both the secret and duplicate checks.
- Approval renders niucode's structured change preview (hunks, capped at 80 lines) before asking.
- Memory is sent to the provider on every turn, so the leak surface is larger than a normal file.
- Project knowledge stays in the repo's `AGENTS.md`; niuse never edits your repositories.
- `NIUSE_MEMORY=off` disables injection and candidates, for A/B runs and for recovering trust.
- `niuse doctor` checks home permissions, identity, credentials, bwrap, and the pinned niucode revision without calling a model.
- `metrics.jsonl` records local counters only: turn outcomes, candidate decisions, tags, and ids. Memory text never enters it.

## Explicit non-goals

A daemon, a web UI, WeChat / Feishu bridges, scheduled push notifications, vector search, full session indexing, automatic repo edits, and multi-user support. The TUI is opt-in until ADR-0004's gate.

[ADR-0005](adr/0005-working-companion.md) widens the product scope: project awareness, briefs, goals, delegation, and connectors are allowed when they serve one person and stay terminal-first. The test for every proposal is still: **does this help the companion finish real work without making the system harder to trust?**
