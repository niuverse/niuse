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

### M5 · Harness-native companion (active)

**Goal:** make the existing harness the companion's default working surface
instead of maintaining a parallel front end.

**Exit**

- `niuse tui` reaches approval-preview and usage parity with the REPL contract.
- The product commands, `/brief`, `/goal`, `/plan`, and skills are discoverable
  in the TUI and covered by tests.
- Ten real TUI dogfood days are recorded, one real task per day.
- The renderer is retired only after that gate, per ADR-0004.

**Cards:** M5-001 done; M5-002 is 2/10 TUI days.

### M6 · Compounding context (done)

**Goal:** make what the companion knows visible and useful at the moment work
starts, without adding a database.

**Exit**

- A startup brief surfaces overdue promises, review queues, and stale memory.
- A project registry answers "what was I doing, and where?" across repositories.
- A session digest turns a finished thread into a journal entry with evidence.
- Memory has an explicit on-demand recall path beyond the injected budget.

**Cards:** M6-001 through M6-013; the phase is complete, digest stays opt-in.

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

### M8 · Connected agent (done)

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
test. [ADR-0034](https://github.com/niuverse/niucode/blob/main/docs/adr/0034-mutating-mcp.md)
is accepted and implemented: `write_tools`, `--allow-mcp-write`, per-call
approval, shared write lock, no retry. niuse exposes the flag, mounts writes
only with an interactive terminal, ships an opt-in `create_note` notes tool,
and the TUI PTY proves the preview plus deny/allow. M8 is complete.

### M9 · Proof (planned)

**Goal:** measure whether the companion is actually more useful, and keep it
that way.

**Exit**

- An offline personal-agent suite covers memory recall, briefs, goals, and
  delegation.
- A memory-on/off A/B case shows a behavioural difference.
- Regressions in dogfood metrics block a release.

**Cards:** M9-001 through M9-006 are done: briefs, goals, memory A/B, recall, MCP boundaries, the dogfood trend gate, prompt-size budgets, token/cache signals, a combined two-pack workflow, multi-turn plan execution, and failed-verification recovery are covered.

## Current sprint

One card at a time; a round is not done until the card, docs, dashboard, tests,
and commit agree.

| Order | Card | Status | Why now |
|---|---|---|---|
| 1 | M6-001 startup brief | done | Makes existing context actionable without new architecture |
| 2 | M6-002 project registry | done | Answers "where was I?" before planning work |
| 3 | M5-001 TUI parity | done | PTY deny/allow and event-log `/cost` verified |
| 4 | M5-002 TUI dogfood | ready | Product proof for the new default |
| 5 | M6-003 session digest | done | 10/10 reviews; opt-in by choice |
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
| 26 | M9-006 verification failure | done | A failed check is visible and blocks done |
| 27 | M7-009 TUI plan PTY | done | The full TUI gate runs in a real terminal |
| 28 | M2-094 doctor dogfood | done | The ten-day gate is visible while using the product |
| 29 | M2-095 dogfood checker | done | The gate cannot be claimed from a malformed table |
| 30 | M2-096 dogfood command | done | The row comes from metrics, not hand arithmetic |
| 31 | M2-097 weekly report | done | One read-only view for the weekly review |
| 32 | M9-007 prompt prefix | done | Cacheable bytes are pinned; protocol heading deduped |
| 33 | M2-098 report commands | done | The weekly review is available in-session |
| 34 | M9-008 link check | done | Docs cannot rot into broken relative links |
| 35 | M9-009 board check | done | Milestone and card states cannot contradict |
| 36 | M9-010 README contract | done | Docs commands are checked against the binary |
| 37 | M9-011 README flags | done | Documented flags must exist on the command |
| 38 | M2-001 real days | doing | 2/10; 3 cited ids, A/B, disagreement |
| 39 | M6-003 real digest review | done | 10/10 dry-runs reviewed; opt-in stays |
| 40 | M6-006 digest no-change | done | NONE exits cleanly; malformed still fails |
| 41 | M6-007 digest gate | done | YES/NO pre-check stops activity summaries |
| 42 | M6-008 gate old loops | done | Old-loop reminders are not new issues |
| 43 | M6-009 gate reasons | done | Every YES/NO gate verdict is explained |
| 44 | M6-010 digest evidence | done | Verdicts and step tags reach the digest |
| 45 | M7-005 real gate | done | A real model refused to close an unverified step |
| 46 | M6-011 failed-check digest | done | Failure then repair is journal-worthy |
| 47 | M6-012 unresolved failure | done | A failed check is never written as success |
| 48 | M6-013 restated memory | done | Restating memory is not a new change |
| 49 | M5-002 TUI gate | doing | 2/10 TUI days; two gates may overlap |
| 50 | M3-004 publish state | done | Docs match crates.io and the MCP blocker |
| 51 | niucode M8-003 MCP preflight | ready | Missing crate is packageable; upload awaits human |
| 52 | M5-004 TUI clear session | done | /new archives and resets thread.jsonl |
| 53 | M5-005 TUI reset session | done | /reset gets the same isolation proof |
| 54 | niucode M6-028 session notify | done | /resume and /fork update the hook pointer |
| 55 | M5-006 TUI resume pointer | done | /verify after /resume hits the right thread |
| 56 | ADR-0007 active thread | done | /resume is session-scoped; one active line |
| 57 | M5-008 TUI fork pointer | done | /verify after /fork hits the destination |
| 58 | M5-009 TUI memory prompt | done | Accept/discard works through a real terminal |
| 59 | M5-010 TUI turn metrics | done | Real TUI turns reach stats and dogfood |
| 60 | M2-099 distinct citations | done | Three different ids, not three rows |
| 61 | M5-011 TUI command compose | done | Brief, report, and steps run back-to-back |
| 62 | M3-006 niuse 0.1.12 | done | Published; package and clean install pass |
| 63 | M8-008 write MCP surface | done | Flag, write pack, TUI deny/allow proven |
| 64 | M8-009 evidence mounts | done | /evidence shows servers, writers, hashes |
| 65 | M3-007 release runbook | done | One reviewable 0.1.12 upload step |
| 66 | niucode M9-001 rich text | done | Inline Markdown and boxed tables in the TUI |
| 67 | niucode M9-002 math | done | Common TeX falls back to Unicode |
| 68 | niucode M9-003 lists | done | Bullets, tasks, and quotes render |
| 69 | niucode M9-004 fences | done | Fences become a language label |
| 70 | niucode M9-005 display math | done | $$ blocks lose fences and use Unicode |
| 71 | niucode M9-006 motion | done | Streaming caret and per-tool elapsed |
| 72 | niucode M9-007 web viewer | done | Loopback read-only session viewer |
| 73 | niucode M9-008 web SSE | done | Live event tail in the browser |
| 74 | niucode M9-009 web prompt | done | Read-only browser chat is live |
| 75 | niucode M9-010 web streaming | done | Deltas arrive before turn end |
| 76 | niucode M9-011 web approvals | done | Browser Allow/Deny releases the turn |
| 77 | niucode M9-012 web Markdown | done | HTML, tables, safe links in the browser |
| 78 | niucode M9-013 web MathML | done | Native MathML, no remote assets |
| 79 | M10-001 niuse web | done | Product prompt over niucode web |
| 80 | M10-002 web sidebar | done | Identity/memory/plan context in the browser |
| 81 | niucode M9-017 web timer | done | Phase + elapsed chip, collapsible context |
| 82 | M10-003 web report | done | Weekly report/dogfood in the sidebar |
| 83 | niucode M9-018 lists/rules | done | Ordered lists and horizontal rules in TUI |
| 84 | niucode M9-019 table chrome | done | Bold headers, dim borders |
| 85 | niucode M9-020 table cells | done | Inline styles survive cell layout |
| 86 | niucode M9-021 code tokens | done | Keywords/strings/comments/numbers |
| 87 | niucode M9-022 web code | done | Highlighted code in browser HTML |
| 88 | niucode M9-023 web streaming MD | done | Markdown forms during the turn |
| 89 | niucode M9-024 approval diff | done | Structured diff cards, not JSON |
| 90 | niucode M9-025 tool cards | done | Collapsible call/result cards |
| 91 | niucode M9-026 code state | done | Block comments span lines |
| 92 | niucode M9-027 tool markers | done | One marker per tool type per turn |
| 93 | niucode M9-028 triple strings | done | Docstrings span lines |
| 94 | niucode M9-029 sidebar filter | done | Collapsible/filterable context |
| 95 | M10-004 live web context | done | Context re-read per request |
| 96 | niucode M9-031 web composer | done | Enter sends, Shift+Enter newline |
| 97 | niucode M9-032 prompt history | done | Local ↑/↓ recall |
| 98 | niucode M9-033 code copy | done | Copy buttons on web code |
| 99 | niucode M9-034 context event | done | context_changed after turns |
| 100 | niucode M9-035 web commands | done | /help /status /report locally |
| 101 | niucode M9-036 unknown slash | done | Reject locally, no model turn |
| 102 | niucode M9-037 command menu | done | Filter/complete web slash commands |
| 103 | niucode M9-038 web clear | done | View-only clear, log unchanged |
| 104 | niucode M9-039 web usage | done | Page-local in/out token chip |
| 105 | niucode M9-040 web export | done | Download session Markdown |
| 106 | niucode M9-041 web search | done | Filter transcript locally |
| 107 | niucode M9-042 MSRV/package | done | 1.88 clean, package verifies |
| 108 | niucode M9-043 web new session | done | Fresh JSONL via /api/new |
| 109 | M10-005 web new thread | done | Archive thread, recreate thread.jsonl |
| 110 | niucode M9-044 web autoscroll | done | Follow only near bottom |
| 111 | niucode M9-045 web theme | done | Light mode + reduced motion |
| 112 | niucode M9-046 web tests | done | Export/new-session HTTP contracts |
| 113 | niucode M9-047 heading levels | done | H1/H2/H3+ emphasis |
| 114 | M10-006 web threads | done | Archived chapters in sidebar |
| 115 | M10-007 web switch | done | Move archived chapter to active thread |
| 116 | niucode M9-050 session event | done | All tabs reload on switch |
| 117 | niucode M9-051 tool timing | done | Aggregate per-tool durations |
| 118 | niucode M9-052 high contrast | done | Third accessible theme |
| 119 | niucode M9-053 web a11y | done | Labels, log role, OS theme defaults |
| 120 | niucode M9-054 DOM test | done | Unique ids, resolvable references |
| 121 | niucode M9-055 web smoke | done | Headless DOM smoke test |
| 122 | niucode M9-056 chip visual fix | done | Readable header buttons |
| 123 | niucode visual audit | done | Populated transcript screenshots |
| 124 | niucode M9-057 contrast/mobile | done | Deny readable, mobile header, /export |
| 125 | niucode M9-058 TUI motion | done | Reduced-motion env fallback |
| 126 | niucode M9-059 viewport rules | done | Rules fill the terminal width |
| 127 | niucode M9-060 session approval | done | Allow session in web |
| 128 | niucode M9-061 approval countdown | done | 5:00 deadline, disabled at zero |
| 129 | niucode M9-062 tool approval | done | Allow only the requested tool |
| 130 | niucode M9-063 image block | done | No remote image fetches |
| 131 | niucode M9-064 link hardening | done | New-tab links, blocked schemes |
| 132 | niucode M9-065 href allowlist | done | Parsed schemes, mixed-case safe |
| 133 | niucode M9-066 loopback guards | done | Host/Origin checks, CSP |
| 134 | niucode web threat model | done | Controls, tests, residual risks |
| 135 | niucode M9-067 local token | done | Random URL token + HttpOnly cookie |
| 136 | niucode M9-068 token page | done | Explain missing token in browser |
| 137 | niucode M9-069 mobile sidebar | done | Overlay toggle below 980px |
| 138 | niucode M9-070 mobile overlay | done | Backdrop, Escape, focus |
| 139 | niucode M9-071 web keys | done | Ctrl+K, Ctrl+/, Tab trap |
| 140 | niucode M9-072 export calls | done | Tool args in Markdown export |
| 141 | niucode M9-073 copy tools | done | Copy tool output/commands |
| 142 | niucode M9-074 message copy | done | Copy full user/assistant message |
| 143 | M10-008 release preflight | done | Clean trees, pin, both repos, no publish |
| 144 | niucode M9-075 task lists | done | Web checkboxes for - [ ]/- [x] |
| 145 | niucode M9-076 footnotes | done | Web footnote references |

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
