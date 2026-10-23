+++
id = "status"
title = "Status"
kind = "doc"
status = "living"
order = 1
summary = "Entry point after the autonomous iteration: what works, what is pinned, and what waits for a human decision."
+++

## Snapshot · 2026-10-23

niuse is runnable end to end: seven Markdown stores, an interactive REPL with line editing, history, completion, multi-line input, confirmation and conflict handling, the in-band protocol, a read-only startup brief, bootstrap from history (memory candidates and a user profile), JSON views for every store, one-shot session listing, colored approval diffs, `doctor`, and acceptance metrics. The declared MSRV is 1.88; the full test suite and the optional `tui` feature pass under it, and a weekly workflow checks both.

Both repositories are green. The board at <https://niuverse.github.io/niuse/> is deployed from a single generated file; source docs are not published. niuse builds against the niucode revision in `niucode.rev`. The long-running direction is in [roadmap](roadmap.md): harness-native companion, compounding context, long-horizon work, connected tools, and proof.

## Verify

```sh
./scripts/check.sh              # fmt, clippy, docs, tests, package, dashboard
./scripts/check-niucode-drift.sh
cargo xtask dashboard --check
cargo +1.88.0 check --workspace --all-targets   # MSRV, also run weekly
```

## Works

- `niuse` REPL: streaming, slash commands, line editing and history, Tab completion for commands/ids/paths, backslash continuation, `/new` chapters, `/cd`, `/model`, `/cost`, `/threads`, `/sessions`, `/resume`, `/export`, `/brief`, `/plan`, candidate confirm/edit/skip, conflict replace/keep.
- Brief: `niuse brief [--json]`, `/brief` in the REPL and TUI, and a compact actionable block after the banner; no provider call and `NIUSE_BRIEF=off` restores the quiet start.
- Projects: `niuse projects [--limit N] [--all] [--json]` reads niucode's recorded workspaces, sorts by recent activity, hides missing and temporary test checkouts by default, and shows the current project in `status`.
- Digest: `niuse digest [--thread PATH] [--dry-run] [--limit N]` compresses recent user/assistant prose into at most five journal bullets with one tool-free model call; tool output never enters the prompt, secrets refuse the whole write, and the path stays opt-in until ten real digests are reviewed.
- `niuse tui` (optional feature): niucode's TUI with `CompanionHooks`; protocol stripping, journal/loops, memory accept/edit/replace/keep, read-only commands, `/brief`, `/goals`, `/plan`, `/recall`, `/export`, `y`/`n`/`s` approval keys, and `/cost` read from the thread event log.
- MCP: every path mounts read-only servers from `NIUSE_MCP_CONFIG` or `~/.niucode/mcp.toml`; a broken manifest warns and disables MCP for that run. The notes pack (`packs/notes`) exposes search/read/list over `NOTES_DIR` and refuses path escapes; the calendar pack (`packs/calendar`) lists and searches one ICS file without network or timezone conversion. `niuse mcp init` installs both packs, writes a read-only manifest, and merges new servers into an existing manifest by name without losing comments; `--force` replaces it. A PTY test drives the TUI mount and an MCP tool call through the replay provider.
- Memory: search including expired entries, kind filter, kind counts, show/edit, review with metrics, prune with backup and freed-byte report, duplicate guard, secret scan, soft expiry.
- Evidence: `niuse evidence [--thread PATH] [--json]` and `/evidence` report tool calls, write/edit paths, test-like bash checks, recorded verifications, delegated subagent runs (including unfinished ones), failures, and denied approvals without claiming correctness. `niuse verify -- <command>` and `/verify <command>` run a check and append a `verification` event.
- Recall: `niuse recall <query> [--json]` and `/recall` search memory (including expired), journal, loops, and goals together; the agent can also call the read-only `memory_search` tool during a turn, and the protocol tells it to request `/recall` when no tool is available.
- Dates: journal, memory, and stats day labels follow the local calendar; event timestamps stay UTC.
- Journal and loops: model-written blocks plus manual entry and manual promises.
- Plan: `plan.md` holds human-owned steps with `p-N` labels; open steps are injected after goals and before memory. `niuse plan list|add|done` edits them, and `done` refuses without a passing `verification` event tagged with that step. The REPL and TUI expose `/steps` (list/add/done); the TUI also has `/verify p-N <command>` over the same gate, with a PTY regression for the full flow. `niuse evidence` shows each step's verification status.
- Goals: `goals.md` holds human-owned outcomes with `g-` ids; `niuse goals list|add|done` and `/goals` edit them, at most seven open, secret-scanned, and injected after user and before memory.
- Bootstrap: dry-run preview, candidate merge into the inbox, optional `user.md` draft.
- Sessions: derived thread titles, `threads` and `sessions` listings, Markdown export with a title, JSON output.
- Machine output: JSON for memory (list/search/show), journal, threads, sessions, loops, status, doctor, stats, config effective, identity, and user; `stats` totals and trends include token usage, cache-hit percent, and tokens per turn.
- Operations: pinned niucode revision, contract tests, nightly main check, doctor, acceptance metrics.
- Distribution: `niuse`/`niuse-core` **0.1.12** and the eight `niucode-*` crates **0.1.6** are published. `cargo package -p niuse` passes against the registry; a clean-root `cargo install niuse --version 0.1.12 --locked` reports `niuse 0.1.12`. M3 is done; the [0.1.12 release page](release-0.1.12.md) records the preflight and the retry.

## Stage

[Stage report · 2026-10-23](stage.md) records what is proven, the open gates,
and the decisions waiting on a human.

## Waiting on you

1. **Dogfood** — ten working days recorded in [dogfood](dogfood.md); started 2026-10-22 at 1/10 with a real read-only `exec` turn (README.md:106 citation, 11k in / 76 out / 3.5k cached) a real MCP turn (`mcp__notes__search_notes` found `launch.md`, 5k in / 95 out / 3.1k cached), a real TUI turn (`391`, 13k in / 2 out), a real approved write in a scratch workspace (3 steps, 7.1k in / 279 out / 6.3k cached), a real `NIUSE_MEMORY=off` A/B (on cited `m-3d0b`; off said `NO_INJECTED_MEMORY`), and one recorded disagreement: the companion argued against a third RSS pack and I dropped it. 2026-10-23 is 2/10: a real TUI task read `DAY_TWO_C.md` (27k in / 47 out / 13.4k cached) after the missing TUI `on_turn_end` metric was fixed. `niuse dogfood --file docs/dogfood.md --note "..."` (or `./scripts/dogfood.sh "note"`) writes the day's row from metrics; `niuse report --days 7` (also `/report [days]` in the REPL and TUI) assembles turns, tokens/cache, open goals/promises/plan, and dogfood progress; `niuse doctor` shows the active-day count; `scripts/dogfood-check.sh --complete` checks the table and criteria. Only real use and review can close M2.
2. **TUI dogfood** — ten real days on `niuse tui` for M5-002; this is also the ADR-0004 gate for retiring the REPL renderer.

## Milestones

- **M5 · Harness-native companion — active.** M5-001 is done: configurable `y`/`n`/`s` approval keys, an offline PTY deny/allow test, product commands in TUI help/completion, and `/cost` computed from the thread event log. M5-002 is 2/10 TUI days after real TUI turns on 2026-10-22 (`391`, 13k in / 2 out) and 2026-10-23 (27k in / 47 out / 13.4k cached); eight distinct working days remain. The shared TUI now renders inline Markdown (bold, italic, code, links, headings), pipe tables as box-drawing blocks, common TeX math as Unicode, lists/tasks/quotes, code fences replaced by a language label, display math rendered without `$$` fences, a streaming caret/per-tool timer in the activity footer, and niucode `web --session` serves a loopback-only read-only viewer with a live SSE tail a read-only browser prompt, streaming deltas in the web client, and browser approvals, and server-rendered Markdown and native MathML in the web transcript; the raw event log is unchanged. `/new` (alias of `/clear`) and `/reset` both archive the old thread and reset `thread.jsonl` through niucode's new-session seam; `/resume` and `/fork` now notify the hooks too; PTY tests prove `/verify` after `/resume` and `/fork` writes to the live session, the after-turn memory prompt can accept or discard, and `/brief`, `/report`, and `/steps` compose in one session. Cross-restart active-thread semantics are decided in [ADR-0007](adr/0007-active-thread.md): `/resume` and `/fork` are session-scoped; `thread.jsonl` stays the only active line.
- **M6 · Compounding context — done.** Brief, project registry, unified recall, and `memory_search` landed. The digest reached 10/10 real reviews; `NONE`, old-loop reminders, restated memory, and failed-then-fixed checks are handled, every verdict prints a reason, and the digest stays opt-in by choice.
- **M7 · Long-horizon work — done.** Goals, plan mode, evidence, recorded verification, verification-gated steps, open-plan injection, and delegated-run evidence all landed; a real model respected the gate and refused to close a step without evidence. The next product phase is M8 connected agent and M9 proof.
- **M10 · Web companion — active.** `niuse web --features web` serves the browser surface with the identity/memory/plan system prompt and a product sidebar over niucode's SSE, approval, Markdown, and MathML code. The feature is opt-in so the default binary still does not link niucode. The sidebar shows identity/user/goals/plan/memory/loops plus a bounded 7-day report (including dogfood), alongside model/policy status and a live SSE turn timer. The shared TUI now covers ordered lists, horizontal rules, and bold table headers with dim borders and inline styles inside table cells, plus tokenized code blocks streaming rendered Markdown, readable diff previews in web approvals, collapsible tool cards, and multi-line block-comment highlighting, one compact tool marker per type per TUI turn, triple-quoted strings across code lines, a collapsible/filterable web sidebar, live context refresh from the stores, a multi-line web composer, and local prompt history.

- **M8 · Connected agent — done.** niucode ships the stdio MCP client with explicit trust, timeouts, and metadata; niuse mounts it in the TUI, REPL, and `exec`, and ships notes/calendar packs. ADR-0034 is implemented: `write_tools`, `--allow-mcp-write`, interactive mount gating, per-call approval with a bounded argument preview, shared write lock, no retry, and no subagent inheritance. The notes pack has opt-in `create_note`; driver, agent, and TUI tests cover deny/allow, the TUI PTY proves the preview and both outcomes, and `/evidence` shows each mount's server, write allowlist, and config hash.
- **M9 · Proof** — `./scripts/eval.sh` covers memory-on/off A/B, goal and plan injection, the startup brief, agent recall, the notes and calendar MCP packs, MCP timeout/read-only refusal, prompt-size budgets, a stable cache-prefix invariant, a combined two-pack + plan + verification workflow, multi-turn step execution where closed steps leave the next prompt, and the failed-verification recovery path. `scripts/dogfood-trend.sh` compares two weeks of real use and blocks `publish-niuse.sh` on a regression; `scripts/check-links.py` keeps the docs' relative links honest; `scripts/check-board.py` rejects done milestones with open cards, doing cards in planned milestones, and missing blocked reasons; README CLI examples and flags are checked against clap in the test suite.
- **M2 dogfood** is in progress: 2/10 days, acceptance 100% (1/1), three distinct cited memory ids and one disagreement so far. The real 2026-10-23 retrieval cited `m-f356`, `m-5c68`, and `m-3d0b`; the day is also TUI day 2/10. It still decides whether the memory layer pays for itself.

## Not built

Daemon, web UI, messaging bridges, scheduled push, vector search, full session indexing, automatic repo edits, and multi-user support remain out until a new ADR. The TUI is opt-in, not the default; see [roadmap](roadmap.md) for the order.
