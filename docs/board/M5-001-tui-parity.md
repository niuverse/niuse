+++
id = "M5-001"
title = "TUI approval and usage parity"
status = "blocked"
milestone = "M5"
module = "cli"
size = "M"
priority = "P0"
updated = "2026-10-22"
blocked_reason = "REPL keys (y/n, d=full diff) conflict with the TUI pager (a allow, d deny, s session); decide the shared approval contract upstream first"
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

- [ ] A PTY test denies an approval and asserts the same preview fields and cap
      as the REPL test.
- [ ] `d` shows the full diff without accepting the write.
- [ ] `/cost` and the post-turn usage line agree with the event log.
- [ ] Every listed product command appears in TUI help/completion and returns
      the same data as its REPL counterpart.
- [ ] `./scripts/check.sh` is green, including the `tui` feature.

## Notes

The TUI pager is not missing the diff: it renders a scrollable full-screen diff,
which is better than the REPL's 80-line preview. The real parity question is the
key contract. In the REPL `d` means "show the full diff" and `n` denies; in the
TUI pager `d` denies and `s` allows for the session. A user who learned one UI
can deny a write by accident in the other. Resolve that upstream (a shared key
map or an embedder option) before treating this card as parity.

If the gap is a niucode API issue, upstream it and bump the pin through
`scripts/sync-niucode.sh`; do not fork the TUI.
