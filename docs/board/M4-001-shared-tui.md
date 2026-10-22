+++
id = "M4-001"
title = "Reuse niucode's TUI through a library and hooks"
status = "blocked"
blocked_reason = "Renderer removal waits on ten real TUI dogfood days (M5-002) and ADR-0004."
milestone = "M4"
module = "frontend"
size = "L"
priority = "P0"
updated = "2026-10-22"
depends_on = ["M0-003"]
docs = ["adr-0003", "compat"]
+++

## Scope

- `niucode-cli` exposes a library target: session runtime, TUI entry, renderer, approval UI.
- A small hook surface: augment the system prompt before a turn, observe a finished turn, receive session lifecycle events.
- niuse implements the hooks: identity / memory injection, candidate extraction, journal and loop writes.
- Delete niuse's renderer and its per-turn `AgentConfig` assembly once the shared path is stable.
- Step 1 is done in niucode `aac20a1`: `niucode-cli` exports `run()`, `TurnHooks`, and `NoHooks`.
- Step 2 is done for both entry points: the exec path invokes all three hooks through `run_exec_with_hooks`; the TUI path has `TuiOptions` / `run_tui_with_hooks` and invokes prompt, event, and turn-end through a shared hook cell. `filter_text` adds the streaming seam.
- Renderer migration is analysed in [M4-002](M4-002-renderer-gap.md); the files do not move until the confirmation and command seams exist.

## Acceptance

- [ ] A niuse binary runs niucode's TUI with memory injected and candidates confirmed.
- [ ] niuse no longer owns a renderer or a second terminal input loop.
- [ ] The hook trait is generic: no identity or memory types cross into niucode.
- [ ] Contract tests still pass against the pinned revision.

## Notes

This is the only large piece of duplication left. Keeping two repos is still the decision (ADR-0003); this card removes the reason the boundary felt wrong, without merging the products.
