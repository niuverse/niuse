+++
id = "M5-001"
title = "TUI approval and usage parity"
status = "done"
milestone = "M5"
module = "cli"
size = "M"
priority = "P0"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M4-003"]
docs = ["renderer-migration", "adr-0004", "dogfood"]
+++

## Problem

`niuse tui` already runs the product hooks, but its approval presentation and
usage line are niucode's defaults. The REPL contract is stricter: an 80-line
preview, `d` for the full diff, denial returns the structured preview, and one
session-level usage line feeds `/cost`. Until the two match, the renderer
cannot be retired and the TUI cannot become the default.

## Scope

- Compare the approval path end to end: preview cap, full-diff action, denial
  payload, colour, and cancellation.
- Compare usage accounting: per-run totals, `/cost`, cached tokens, and the
  compact line shown after a turn.
- Make every product command reachable and discoverable in the TUI help and
  slash completion: `/brief`, `/mem`, `/loops`, `/journal`, `/status`, `/cost`,
  `/export`, `/threads`, `/sessions`, `/prompt`, `/identity`, `/user`.
- Record the decision in `renderer-migration.md`; do not delete `render.rs` in
  this card.

## Acceptance

- [x] A PTY test denies an approval with the configured `y`/`n` keys and
      asserts the write is blocked and the preview was visible.
- [x] `/cost` and the post-turn usage line agree with the event log.
- [x] Every listed product command appears in TUI help/completion and returns
      the same data as its REPL counterpart.
- [x] `./scripts/check.sh` is green, including the `tui` feature.

## Notes

The key conflict is resolved upstream: niucode `2d12360` adds
`TuiOptions.approval_keys`, and niuse sets `y`/`n`/`s`. `d` no longer denies on
the niuse path; the pager's full diff is always available with scrolling, so
the 80-line cap is a REPL property, not a TUI requirement.

`tests/tui_approval.py` is the offline PTY proof: it sees the configured footer,
checks that `d` is ignored, denies with `n` and confirms no file, then allows
with `y` and confirms the write. `/cost` now reads the thread event log, the same
source as the REPL, instead of accumulating a second total in the hook.

The renderer stays until M5-002 records ten real TUI days (ADR-0004).

If the gap is a niucode API issue, upstream it and bump the pin through
`scripts/sync-niucode.sh`; do not fork the TUI.
