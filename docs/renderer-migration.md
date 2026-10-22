+++
id = "renderer-migration"
title = "Renderer migration"
kind = "doc"
status = "living"
order = 60
summary = "What niuse's renderer does, which hooks exist, and why the files do not move until confirmation and command seams land."
+++

# Renderer migration

Status: gap analysis, 2026-10-22. No renderer code has moved yet.

## What exists

- **niuse** owns a small renderer: `crates/niuse-cli/src/render.rs` (361 lines) plus the REPL loop in `repl.rs`. It does five jobs:
  - streaming text through `candidate::StreamFilter`, which strips `<<niuse:*>>` blocks;
  - parsing the stripped answer into memory / loop / journal candidates;
  - rendering tool calls, tool results, warnings, compaction, and one usage line;
  - asking y/n/e/s for a candidate and r/k/d for a conflict;
  - approval previews with a unified diff, 80-line cap, and `d` for the full diff.
- **niucode** owns the TUI: `tui.rs` (2 774), `terminal.rs` (480), `live.rs` (460), and `presentation.rs` (246). It owns the alternate screen, inline viewport, composer, history, approvals, diff rendering, markdown-ish theme, details view, and slash commands.

The duplication is real, but it is not one renderer: niuse's candidate confirmation and memory commands are product UI, not terminal plumbing.

## Seams now available (niucode 0.1.5+)

- `run_tui_with_hooks(TuiOptions, Box<dyn TurnHooks>)`.
- `augment_system_prompt` — memory/identity injection.
- `on_event` — observe every runtime `AgentEvent`.
- `filter_text` — mutate a text delta before display; the persisted event is untouched.
- `on_turn_end` — observe the final `RunOutcome`.
- `after_turn` / `on_answer` — ask a numbered question after a successful turn; the host renders it and returns the choice.

`filter_text` (published in niucode 0.1.3) is the first renderer seam: a niuse hook can run `StreamFilter` and strip protocol blocks before the TUI prints them, while capturing the raw deltas for parsing at turn end. The example `hooked_tui.rs` verifies the filter under a PTY.

## Gaps that still block deleting `render.rs`

1. **Candidate confirmation.** `after_turn` provides a generic numbered prompt, and the `tui` feature carries `CompanionHooks` (capture, parse, auto loop/journal, memory accept/discard). `niuse tui` wires it end to end, including conflict replace/keep and `e` edit. Read-only `/mem`, `/loops`, `/journal`, `/status`, `/cost`, `/threads`, `/sessions`, `/export`, `/prompt`, `/identity`, and `/user` work through the command hook and are registered in help/completion; automatic journal/loop writes also record metrics. The TUI's per-turn usage line keeps niucode's shape; `/cost` reads the same thread event log as the REPL.
2. **Product commands.** `TurnHooks::commands` and `TurnHooks::command` now register and dispatch product commands; `/brief`, `/mem`, `/loops`, `/journal`, `/status`, `/cost`, `/export`, `/threads`, `/sessions`, `/prompt`, `/identity`, and `/user` are reachable from the TUI.
3. **Approval presentation.** The TUI pager renders a scrollable full-screen diff, and niucode `2d12360` lets an embedder configure the approval keys; niuse sets `y`/`n`/`s`, so `d` no longer denies. `tests/tui_approval.py` proves deny/allow and that `d` is ignored. The 80-line cap is a REPL property; the pager scrolls instead of truncating.
4. **Usage and `/cost`.** `/cost` now reads the thread event log, so it agrees with the REPL and the log by construction. The per-turn line remains niucode's `turn N · in · out · cache%` shape.
5. **Config semantics.** `TuiOptions` is close to niuse's CLI flags, but niuse's config precedence, `NIUSE_MEMORY=off`, and provider presets still live in niuse.

## Staged plan

1. **Text filter (done).** `filter_text` exists and is verified; niuse can strip protocol blocks while capturing raw text.
2. **Post-turn interaction hook (next).** Add a hook that lets the embedder present a candidate decision through the host UI, or a first-class "candidate" widget/command extension. This is the hard one.
3. **Command registry injection.** Let the embedder add slash commands with help text and completion.
4. **Opt-in TUI mode (prototype done).** `niuse tui` runs under the `tui` feature with `CompanionHooks`; a replay PTY test confirms protocol stripping, the accept/discard question, and the memory write. The rustyline REPL remains the default.
5. **Retire `render.rs` only after** ten real dogfood days on the TUI path, the same bar as M2. Until then the renderer stays; it is not dead code.

## Decision

Do not migrate the renderer by moving files. Extend the hook surface first: a post-turn interaction seam and command registration are cheaper and safer than porting candidate UI into niucode. The two repos stay separate (ADR-0003).

The cost today is one small renderer; the cost of a premature migration is a product UI split across two event loops.
