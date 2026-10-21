+++
id = "plan"
title = "Implementation plan"
kind = "plan"
status = "active"
order = 10
summary = "Fifteen decisions, the five stores, turn lifecycle, write thresholds, milestones, and verifiable acceptance."
+++

## In one sentence

Add a persistent identity and memory layer above niucode so the same agent knows you across projects and sessions. No resident service, no vector search, no automatic edits to your repositories.

## 1. Positioning

niucode owns "can do": loop, tools, sandbox, event log. niuse owns "who": what it remembers, what it promised, and when it should disagree with you.

One test decides whether a feature belongs here: **does it shape an identity, or is it just another capability?** Capabilities stay in niucode or are not built.

## 2. Decisions

| # | Question | Decision |
|---|---|---|
| D1 | Positioning | Cross-project engineering memory |
| D2 | Content | Preferences, decisions, pitfalls; few, high-value, reviewable |
| D3 | Ownership | Global preferences in niuse; project knowledge in the repo's `AGENTS.md` |
| D4 | Writing | The agent proposes, one key confirms; at most one per turn |
| D5 | Reading | Full injection filtered by scope |
| D6 | Architecture | Separate repo, depends on niucode libs, drives turns itself |
| D7 | Storage | `~/.niuse/`, plain Markdown, `0600` |
| D8 | Interface | Interactive REPL; TUI deferred |
| D9 | Expiry | Soft expiry, replace on conflict |
| D10 | Privacy | Secret-pattern blacklist, refuse on match |
| D11 | MVP | Only the global-preference loop |
| D12 | Top risk | Confirmation fatigue |
| D13 | Shape | Identity layer plus resumable session, no resident process |
| D14 | Cold start | Mine history; you define `identity` |
| D15 | Character | An opinionated collaborator |

Rationale for each trade-off lives in [ADR-0001](adr/0001-memory-layer.md) and [ADR-0002](adr/0002-identity-layer.md).

### Admission criteria for memory

A piece of information may enter `memory/` only if all three hold:

1. It is still useful across sessions.
2. It cannot be found by `grep` in the repository.
3. It fits in one sentence a human can review.

Anything else is rejected. This is what keeps "index every session" and "environment facts" out: they look like memory but are really retrieval and configuration.

## 3. Stores and injection

### 3.1 The five stores

```text
~/.niuse/
  identity.md          who it is: name, voice, values, boundaries
  user.md              who you are: goals, rhythm, taste, red lines, current work
  memory/
    preferences.md
    lessons.md         decisions and pitfalls
  journal/
    2026-10.md         experience, one file per month
  loops.md             promises: things it said it would do
```

- UTF-8 Markdown only; directories `0700`, files `0600`.
- `memory/` may be its own git repository for backup and diffs; `journal/` is not committed by default.
- No database. For a single user and under 200 entries, files are the best query engine.

### 3.2 Injection order and budget

Stable before volatile; the system prompt is assembled once per session (never rebuilt mid-session, to avoid a cache miss on every turn):

| Section | Source | Budget | Change rate |
|---|---|---|---|
| Identity | `identity.md` | ≤ 600 tok | very low |
| User | `user.md` | ≤ 600 tok | low |
| Facts | `memory/*.md` | ≤ 2 000 tok | low |
| Promises | `loops.md` | ≤ 300 tok | medium |
| Recent experience | last 3 `journal/` days | ≤ 600 tok | high |

About 4k tokens in total. Over budget, drop the oldest `unverified` facts first, then truncate the journal; never silently drop `identity` or `user`.

### 3.3 Entry format

One memory is one section: machine-parseable and human-readable.

```markdown
### m-7f3a

- Conclusion: Rust repositories use `cargo nextest`, never `cargo test`
- Tags: rust, testing
- Created: 2026-10-21
- Last confirmed: 2026-10-21
- Source: `1792493790852-350850-0`
```

`id` is four random hex digits and is never reused. `Source` points at the niucode session id and is the only audit trail.

`loops.md`:

```markdown
- [ ] l-7f3a Confirm whether niucode supports per-turn cwd (due 2026-10-22)
- [x] l-91cc Cap the bootstrap sample (2026-10-21)
```

## 4. Turn lifecycle

```text
startup
  ├─ read the five stores
  ├─ assemble the system prompt (once)
  └─ restore thread history via project_history

each input
  ├─ run_with_history(config, prompt, history)
  ├─ stream events (own renderer)
  ├─ tail buffer: strip <<niuse:memory>> / <<niuse:loop>> / <<niuse:journal>> blocks
  ├─ candidate? print it → y / n / e
  ├─ accept → write memory (or loops)
  └─ append to the journal buffer

exit / project switch
  └─ journal is already on disk
```

`history` is projected by niucode's `project_history` from session events; there is no second copy of the conversation. Replay, fork, and compaction therefore behave exactly as in niucode, and niuse does not reinvent them.

## 5. Write thresholds

### 5.1 Candidate protocol

No extra model call. The system prompt states that the model may append a block when a turn produced something durable:

```text
<<niuse:memory kind=preference tags=rust,testing>>
Rust repositories use cargo nextest, never cargo test
<</niuse:memory>>
```

Rules:

- At most one block per turn; if there is nothing, say nothing. **Long runs without a candidate are normal.**
- Only `memory` and `loop` and `journal` blocks are recognised.
- After a `memory` rejection, the same tags are not proposed again in that session.
- A parse failure counts as "no candidate": no error, no guessing.

The cost is that the model may miss things. Missing is acceptable; recording something wrong is not — the same value as D9. If dogfooding shows serious under-recording after two weeks, add an offline proposer then, not now.

### 5.2 Confirmation

```text
Memory candidate [global · rust,testing]
  Rust repositories use cargo nextest, never cargo test
  [y] accept   [n] discard   [e] edit   [s] stop proposing this session
```

`e` opens `$EDITOR`. A conflict does not overwrite silently:

```text
Conflicts with m-91cc:
  old: Rust projects use cargo test
  new: Rust repositories use cargo nextest
  [r] replace   [k] keep both   [d] discard
```

### 5.3 Journal

Written automatically, no confirmation, but only under strict rules:

- Three to five lines, only what **differs from last time**: decisions, turning points, open questions, mood shifts.
- No activity logs ("fixed a bug today"), no duplication of `memory/`.
- Events and judgements only, not code detail. Paths may be referenced; code is not copied.
- Scanned for secrets before writing, since transcripts may contain keys.

The journal is not a log; it is compressed shared experience. The test: read it three months later and see whether the trade-off comes back.

### 5.4 Loops

- When it says "I'll check next time" or "I'll follow up", that goes into `loops.md`; otherwise trust erodes.
- Each entry may carry a due date; entries without one surface after two weeks.
- Closing a loop needs evidence (a verification, a conclusion), not a feeling of completion.

## 6. Expiry, conflicts, privacy

### 6.1 Soft expiry

| Age since `last confirmed` | Injection behaviour |
|---|---|
| ≤ 30 days | injected normally |
| 31–90 days | injected, marked `(unverified since YYYY-MM-DD)` |
| > 90 days | not injected automatically; visible via `/mem recall` |

No automatic deletion. Deletion is irreversible and you would never know what it removed. `/mem review` supports bulk re-confirmation and cleanup.

`journal` is exempt from expiry: experience does not become invalid, only old. `loops` uses due dates, not age.

### 6.2 Conflicts

Before writing, a new memory is compared with the existing store:

- normalise (lowercase, strip punctuation), take character bigrams, Jaccard ≥ 0.5 is a likely conflict;
- same tag and similarity ≥ 0.35 also counts;
- a hit opens replace / keep both / discard; never merge automatically.

The 0.5 threshold is a starting point and will be adjusted after two weeks of false positives. No embeddings: at 200 entries the complexity outweighs the benefit.

### 6.3 Privacy

Memory is sent to the provider every turn, so leaked content leaks far more often than in an ordinary file. Every write is scanned; a hit is refused with a reason:

| Pattern | Example |
|---|---|
| `sk-[A-Za-z0-9]{20,}` | OpenAI / DeepSeek key |
| `ghp_` / `github_pat_` | GitHub token |
| `AKIA[0-9A-Z]{16}` | AWS access key |
| `-----BEGIN .* PRIVATE KEY-----` | private key |
| `xox[baprs]-` | Slack token |
| 32+ hex / base64 run | generic key shape |

Scanning is a backstop, not a guarantee. Private projects, client names, and internal addresses still depend on human judgement at confirmation time. `NIUSE_MEMORY=off` disables injection entirely, for A/B runs and trust recovery.

## 7. Cold start

`niuse bootstrap` turns history into `user.md` and memory candidates once:

1. Enumerate `~/.niucode/sessions/*.jsonl`, last 90 days, at most 40; prefer sessions with multiple turns and tool calls.
2. Send only user / assistant text, **dropping tool output**, through one tool-free extraction that returns `{type, text, evidence}`.
3. Deduplicate, group by type, and write `~/.niuse/inbox.md` with the source session id on every line.
4. `niuse review` walks the queue: `y` accept, `n` discard, `e` edit.
5. `identity.md` is not mined: `niuse identity init` asks eight questions, records your answers, and lets you edit them.

Cost estimate: 40 sessions × about 2k input tokens ≈ 100k tokens, one-off.

Mining boundary: only the three kinds from D2. No code facts, no environment configuration, no information about other people.

## 8. Architecture

```text
crates/niuse-cli     REPL, commands, renderer, driver, bootstrap
    │
crates/niuse-core    date / model / store / prompt / candidate / policy
    │
niucode-agent        run_with_history / AgentConfig / project_history
niucode-core         SessionLog / Event
niucode-provider / auth / tools
```

- Two crates: `niuse-core` depends only on `anyhow` and `regex`, and its 35 tests need no API key; `niuse-cli` owns the terminal, provider wiring, and driver.
- The dependency currently points at a sibling checkout via a path. Publishing or moving machines requires a pinned git revision instead.
- A `NiucodeDriver`-style seam (about 200 lines) isolates API churn: config assembly, history projection, event forwarding.
- `niucode-cli` is a binary with no library target, so the MVP does not reuse its TUI or renderer. It writes a minimal renderer and can swap later once upstream exposes a library or a hook.
- No changes to niucode's loop, no memory hook in the kernel. Memory is a product layer; the kernel stays minimal.

## 9. Three assumptions verified in W1

Written down is not verified. Each was tested in the first week; if any had failed, the design would have changed.

1. **Per-turn cwd on one session** — passes: reading in one directory, `/cd /tmp`, reading in another, both succeed in the same `thread.jsonl`, with history rebuilt correctly.
2. **Reliable stripping of trailing candidate blocks in a stream** — passes: the filter covers blocks split across chunks, partial prefixes, false markers, and unterminated blocks; real calls show the block removed and the prose intact.
3. **`niucode-agent`'s public API fits a thin adapter** — passes: `driver.rs` is about 200 lines with nothing to fork or bypass. The only thing written from scratch is the renderer, because `niucode-cli` has no library target.

## 10. Milestones

| Week | Deliverable | Status |
|---|---|---|
| W1 | Three spikes; crate skeleton; five-store reads/writes; prompt assembly; one real turn in the REPL | done |
| W2 | Candidate block and confirmation; journal; loops; soft expiry; secret scan | done (approval preview is still a summary, not a full diff) |
| W3 | `niuse bootstrap`; dogfood; acceptance data | bootstrap and review done; dogfood pending |

## 11. Acceptance

Technical checks passed on 2026-10-21: five-store reads/writes and `0600`, candidate protocol, streaming strip, soft expiry, secret scan, the full `bootstrap → review → memory` chain, per-turn cwd in one session, and `cargo xtask dashboard --check`.

What remains is product acceptance, which requires real use:

- [ ] Ten consecutive working days, at least one real task per day through niuse
- [ ] Candidate acceptance rate ≥ 60% (accepted / proposed)
- [ ] At least 3 cases where it cites a past judgement and the citation is correct
- [ ] At least 1 case where it disagrees and you change your mind
- [ ] Memory store ≤ 50 entries (more means the bar is too low)
- [ ] At least 1 A/B case where `NIUSE_MEMORY=off` produces a different answer

The last item is the anti-placebo check: if turning memory off changes nothing, the layer has no reason to exist.

## 12. Non-goals

Daemon, web UI, WeChat / Feishu bridges, scheduled push, vector search, full session indexing, automatic repo edits, multi-user support, a TUI (for now), a general agent framework.

## 13. Risks

| Risk | Symptom | Mitigation |
|---|---|---|
| Confirmation fatigue (top) | Blind `y` presses, then the feature is switched off | At most one per turn; no repeated tags after rejection; observable acceptance rate; `NIUSE_MEMORY=off` |
| Persona performance | Lines like "I missed you" | Hard rules in `identity.md`; companionship rests on memory, promises, disagreement |
| Journal becomes an activity log | "Fixed a bug today" | Only changes and judgements; monthly review removes empty entries |
| Dependency rot | niucode moves and the build breaks | Pin the revision; thin adapter; upstream a library split if needed |
| Cold-start noise | "User likes clean code" × 20 | Only three kinds; every candidate carries a source; batch review, not accept-all |

## 14. Open questions

- Its name: `niuse` is the project name, not necessarily what you call it; define that in `identity.md`.
- Bootstrap sample size and cost: decide after one real run.
- Whether it may propose changes to `identity.md`: proposal yes, confirmation required.
- Session model beyond a single thread: `/new`, per-project sessions; wait for real usage to expose the need.

## 15. Implementation map and deviations

Implemented in `crates/niuse-core` and `crates/niuse-cli`:

| Plan item | Location |
|---|---|
| Five stores, `0600` | `store.rs`, atomic write plus `chmod` |
| Injection and budgets | `prompt.rs`, stable sections first |
| Protocol blocks and streaming strip | `candidate.rs`, six boundary tests |
| Soft expiry / conflicts / secret scan | `policy.rs` plus `prompt.rs` |
| REPL, confirmation, `/mem` and friends | `repl.rs`, `commands.rs` |
| Session resume and per-turn cwd | `driver.rs` |
| Cold start | `bootstrap.rs` (bootstrap + review) |
| Architecture dashboard | `crates/xtask` plus `docs/dashboard/project.json` |
| Memory kill switch | `NIUSE_MEMORY=off` in `repl.rs` |

Three deviations from the plan:

1. **A third protocol block, `journal`.** The plan had memory and loop only; to avoid an extra model call, the main model emits a journal block and niuse writes it automatically.
2. **`exec` never writes memory.** It prints candidates to stderr so scripted calls cannot change the store silently.
3. **Approval shows a summary and +/- counts.** A structured unified diff is deferred; the security boundary is unchanged.
