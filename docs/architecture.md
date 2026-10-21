+++
id = "architecture"
title = "Architecture"
kind = "doc"
status = "living"
order = 5
summary = "niuse boundaries, the five stores, turn data flow, write thresholds, and explicit non-goals."
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

Only `driver.rs` touches `niucode-agent` directly. Everything else is either pure domain logic or only knows `niuse-core` types. When niucode upgrades, the required changes are confined to one file.

niuse does not fork niucode and does not add a memory hook to its loop. Memory is a product layer; the kernel stays minimal.

## Modules

| Module | Layer | Responsibility |
|---|---|---|
| `cli` | Interface | flags, subcommands, offline commands, wiring |
| `repl` | Interface | conversation, slash commands, candidate confirmation, rendering |
| `prompt` | Engine | stable-prefix system prompt and budgets |
| `protocol` | Engine | in-band block parsing and streaming stripping |
| `policy` | Engine | soft expiry, conflicts, secret scanning |
| `driver` | Adapters | niucode boundary, sessions, per-turn cwd |
| `bootstrap` | Adapters | old-session mining and the inbox |
| `stores` | State | five-store load / save and permissions |
| `dashboard` | Engineering | offline board generation |

## Turn data flow

```text
startup
  ├─ read identity / user / memory / loops / journal
  ├─ compose() assembles the system prompt (once per session)
  └─ project_history(thread.jsonl) restores history

user input
  ├─ run_with_history(config, prompt, history)
  ├─ AgentEvent → Renderer (TextDelta goes through StreamFilter)
  ├─ trailing buffer strips <<niuse:*>> blocks
  ├─ journal / loop: written automatically
  ├─ memory: print candidate → y / n / e / s
  └─ re-project history, next turn
```

There is no second copy of the conversation. `history` is projected from the event log by niucode's `project_history`, so compaction, replay, and fork behave exactly as they do in niucode.

## The five stores and injection

```text
~/.niuse/
  identity.md      who it is (human-written; it may only propose)
  user.md          who you are (bootstrapped, human-reviewed)
  memory/          facts: preferences, decisions, pitfalls (confirmed, expiring)
  journal/         experience: one file per month, merged per day (automatic)
  loops.md         promises (automatic, optional due date)
  thread.jsonl     the relationship thread
  inbox.md         bootstrap review queue
```

Injection order is fixed, stable before volatile:

| Section | Budget | Change rate |
|---|---|---|
| persona | constant | never |
| identity | ≤ 600 tok | very low |
| user | ≤ 600 tok | low |
| memory | ≤ 2 000 tok | low |
| protocol | constant | never |
| today | one line | daily |
| loops | ≤ 300 tok | medium |
| journal | ≤ 600 tok | high |

The first five sections form the stable cache prefix; only content after `today` can vary per session. The system prompt is assembled once and is not rebuilt mid-session.

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

At most one `memory` block per turn; after a rejection, the same tags are not proposed again in that session. `journal` records changes, not activity logs.

## Safety and privacy

- `~/.niuse` directories are `0700`, files are `0600`; writes go to a temporary file and are renamed.
- Secret patterns are scanned before every write; a match is refused, and the CLI requires `--force` to override.
- Memory is sent to the provider on every turn, so the leak surface is larger than a normal file.
- Project knowledge stays in the repo's `AGENTS.md`; niuse never edits your repositories.
- `NIUSE_MEMORY=off` disables injection and candidates, for A/B runs and for recovering trust.

## Explicit non-goals

A daemon, a web UI, WeChat / Feishu bridges, scheduled push notifications, vector search, full session indexing, automatic repo edits, multi-user support, a TUI (for now), or a general agent framework.

There is one test for every proposal: **does this shape an identity, or is it just another capability?**
