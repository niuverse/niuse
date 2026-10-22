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

## Seams now available (niucode main, 3ec224d)

- `run_tui_with_hooks(TuiOptions, Box<dyn TurnHooks>)`.
- `augment_system_prompt` — memory/identity injection.
- `on_event` — observe every runtime `AgentEvent`.
- `filter_text` — mutate a text delta before display; the persisted event is untouched.
- `on_turn_end` — observe the final `RunOutcome`.

`filter_text` (on main, not yet in a published crate) is the first renderer seam: a niuse hook can run `StreamFilter` and strip protocol blocks before the TUI prints them, while capturing the raw deltas for parsing at turn end. The example `hooked_tui.rs` verifies the filter under a PTY.

## Gaps that still block deleting `render.rs`

1. **Candidate confirmation.** After `on_turn_end`, the embedder can parse the captured raw text, but the TUI gives no way to ask "accept / edit / discard" inside its event loop. Printing to stdout from a hook would fight raw mode and the composer.
2. **Product commands.** `/mem`, `/remember`, `/forget`, `/journal`, `/loops`, `/review`, and `/export` are niuse commands. The TUI's command registry has no embedder extension point; `run_tui_with_hooks` cannot register them.
3. **Approval presentation.** The TUI already has a good approval UI, but niuse's preview contract (80-line cap, `d` full diff, denial preview returned to the model) is part of its acceptance. Reusing the TUI changes that contract and needs an explicit compatibility test.
4. **Usage and `/cost`.** niuse accumulates usage across a session and prints one compact line; the TUI prints its own turn metrics. The numbers exist, but the presentation and `/cost` command need an adapter.
5. **Config semantics.** `TuiOptions` is close to niuse's CLI flags, but niuse's config precedence, `NIUSE_MEMORY=off`, and provider presets still live in niuse.

## Staged plan

1. **Text filter (done).** `filter_text` exists and is verified; niuse can strip protocol blocks while capturing raw text.
2. **Post-turn interaction hook (next).** Add a hook that lets the embedder present a candidate decision through the host UI, or a first-class "candidate" widget/command extension. This is the hard one.
3. **Command registry injection.** Let the embedder add slash commands with help text and completion.
4. **Opt-in TUI mode.** Prototype `niuse --tui` behind a flag, with niuse hooks and a contract test for approval and candidate confirmation. Keep the rustyline REPL as the fallback.
5. **Retire `render.rs` only after** ten real dogfood days on the TUI path, the same bar as M2. Until then the renderer stays; it is not dead code.

## Decision

Do not migrate the renderer by moving files. Extend the hook surface first: a post-turn interaction seam and command registration are cheaper and safer than porting candidate UI into niucode. The two repos stay separate (ADR-0003).

The cost today is one small renderer; the cost of a premature migration is a product UI split across two event loops.
