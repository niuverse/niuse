+++
id = "roadmap"
title = "Roadmap: from memory layer to working companion"
kind = "plan"
status = "active"
order = 11
summary = "M5–M9: a harness-native companion, compounding context, long-horizon work, connected tools, and proof."
+++

## North star

A personal engineering agent that knows the person, the projects, and the work in
flight; it can plan, act, delegate, and return evidence. It still runs in the
terminal, stores what it knows as files, and keeps the kernel generic.

The test for a feature is no longer only "does it shape an identity?". It is
also: **does it help the companion finish real work without making the system
harder to trust?**

## Where we are

- Identity, memory, loops, journal, soft expiry, conflicts, secret scanning, and
  a resumable thread work end to end.
- The REPL is the default. `niuse tui` exists behind the `tui` feature and
  already runs niucode's goals, plans, subagents, skills, resume picker, and
  permission commands through `CompanionHooks`.
- niucode 0.1.5 exposes the library, the hook surface, the command registry,
  `/plan` and `/goal`, subagents, skills, and the extension host.
- The product gate is still dogfood; the old front end is still the default
  until ADR-0004's ten-day TUI gate is met.

## The split

| Layer | Owns | Does not own |
|---|---|---|
| niucode | loop, event log, providers, tools, sandbox, approvals, goals runtime, subagents, extension/MCP seams | personal memory, identity, product commands, personal data |
| niuse | identity, user model, memory, journal, loops, project context, product UX, personal tool packs | a second loop, a second event log, a second durable store |

When a capability is generic, build it in niucode first and consume it through
the pinned revision. When it is personal, keep it in niuse and test it offline.

## Phases

The board allows one active milestone, and that slot stays with **M2 · Dogfood**
until the memory layer proves itself. M5–M9 are implementation tracks: their
cards can be `ready` or `done` while the gate is open. This is deliberate — the
product keeps moving, but the board does not pretend the proof is finished.

### M5 · Harness-native companion (next)

**Goal:** make the existing harness the companion's default working surface
instead of maintaining a parallel front end.

**Exit**

- `niuse tui` reaches approval-preview and usage parity with the REPL contract.
- The product commands, `/brief`, `/goal`, `/plan`, and skills are discoverable
  in the TUI and covered by tests.
- Ten real TUI dogfood days are recorded, one real task per day.
- The renderer is retired only after that gate, per ADR-0004.

**Cards:** M5-001, M5-002.

### M6 · Compounding context (current track)

**Goal:** make what the companion knows visible and useful at the moment work
starts, without adding a database.

**Exit**

- A startup brief surfaces overdue promises, review queues, and stale memory.
- A project registry answers "what was I doing, and where?" across repositories.
- A session digest turns a finished thread into a journal entry with evidence.
- Memory has an explicit on-demand recall path beyond the injected budget.

**Cards:** M6-001, M6-002, M6-003, M6-004, M6-005.

### M7 · Long-horizon work (done)

**Goal:** let one request become a checked sequence of work that survives a
session boundary.

**Exit**

- Product-level goals persist across sessions and are injected with their state.
- A plan → execute → verify loop records each step and its evidence.
- Delegation to subagents has a budget, a scope, and a summary the parent can
  audit.
- A three-day goal completes without re-explaining the task.

**Cards:** M7-001 through M7-007; the phase is complete.

### M8 · Connected agent (planned)

**Goal:** reach personal tools and data through a standard, trust-bounded
connector instead of one-off shell commands.

**Exit**

- niucode ships a stdio MCP client with trust, timeouts, and tool metadata.
- niuse ships at least one personal pack (for example notes or calendar) behind
  least privilege.
- A connector cannot read credentials, escape its declared roots, or write
  without approval.

**Cards:** niucode M8-001 is done: read-only stdio v1 with explicit trust,
config mounting in `exec`/TUI, and offline tests. niuse mounts the host in the
TUI, REPL, and `exec`, ships read-only notes and calendar packs, installs both
with `niuse mcp init` (which merges safely), and covers the TUI mount with a PTY
test. Mutating MCP is
proposed in [ADR-0034](https://github.com/niuverse/niucode/blob/main/docs/adr/0034-mutating-mcp.md)
and blocked on human sign-off; do not implement it before that.

### M9 · Proof (planned)

**Goal:** measure whether the companion is actually more useful, and keep it
that way.

**Exit**

- An offline personal-agent suite covers memory recall, briefs, goals, and
  delegation.
- A memory-on/off A/B case shows a behavioural difference.
- Regressions in dogfood metrics block a release.

**Cards:** M9-001 through M9-005 are done: briefs, goals, memory A/B, recall, MCP boundaries, the dogfood trend gate, prompt-size budgets, token/cache signals, a combined two-pack workflow, and multi-turn plan execution are covered.

## Current sprint

One card at a time; a round is not done until the card, docs, dashboard, tests,
and commit agree.

| Order | Card | Status | Why now |
|---|---|---|---|
| 1 | M6-001 startup brief | done | Makes existing context actionable without new architecture |
| 2 | M6-002 project registry | done | Answers "where was I?" before planning work |
| 3 | M5-001 TUI parity | done | PTY deny/allow and event-log `/cost` verified |
| 4 | M5-002 TUI dogfood | ready | Product proof for the new default |
| 5 | M6-003 session digest | review | Turns finished work into reusable experience |
| 6 | M6-004 unified recall | done | Human path; agent tool follows |
| 7 | M6-005 agent recall | done | Read-only `memory_search` through niucode's embedder seam |
| 8 | M7-001 persistent goals | done | Human-owned goals injected across sessions |
| 9 | M7-002 REPL plan mode | done | Safe read-only planning without the TUI |
| 10 | M8-003 notes pack | done | First real personal MCP connector |
| 11 | M9-001 offline eval | done | Named regressions + dogfood trend gate |
| 12 | M7-003 thread evidence | done | Shows what ran, not just what was claimed |
| 13 | M7-004 recorded verification | done | Human-run check becomes durable evidence |
| 14 | M7-005 plan ledger | done | Steps close only after step-scoped verification |
| 15 | M7-006 plan prompt | done | Agent sees only open steps and the verify rule |
| 16 | M7-007 delegation evidence | done | Delegated runs are visible, unfinished ones are not success |
| 17 | M9-002 prompt budget | done | Fat stores cannot silently inflate every turn |
| 18 | M9-003 cost signals | done | Cache hit and tokens/turn are visible, not guessed |
| 19 | M8-004 calendar pack | done | A second personal source reuses the MCP seam |
| 20 | M8-005 mcp init | done | Two-minute setup for both read-only packs |
| 21 | M8-006 mcp merge | done | Adding a pack does not erase the trust file |
| 22 | M8-007 TUI MCP PTY | done | The last untested MCP path is covered |
| 23 | M9-004 personal workflow | done | Packs + plan + verification compose |
| 24 | M7-008 TUI plan/verify | done | The ledger is usable without leaving the TUI |
| 25 | M9-005 plan execution | done | Closed steps leave the next prompt |

## The round ritual

1. Read this file, `status`, and the chosen card.
2. Implement the smallest complete behaviour; prefer a pure core function and a
   thin CLI/TUI adapter.
3. Add the focused test the card promises; keep all tests offline.
4. Update the card's status, `updated`, and `completed`; update `status` when the
   phase or the next action changes.
5. Run `cargo xtask dashboard` and include the regenerated board.
6. Run `./scripts/check.sh`; commit and push only when it is green.
7. Leave the next card and the next concrete action in `status` for the next
   round.

A round that only rewrites the plan does not count as progress unless it
unblocks a card.

## Guardrails

- No daemon, web UI, scheduled push, vector index, or automatic repository edit
  without a new ADR.
- No second durable store. The event log and the Markdown stores stay the truth.
- No capability without a permission story: approval, sandbox, secret scan, or
  declared roots.
- No silent memory write and no silent external write.
- Kernel changes are generic and tested in niucode first; niuse never forks
  niucode files.
- If the same blocker survives three rounds, mark it blocked with the concrete
  condition instead of looping.

## Later, with an ADR

Background scheduling and notifications; a local web surface; vector or
embedding recall; multi-user support; automatic edits to user repositories.
