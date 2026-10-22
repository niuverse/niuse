+++
id = "M4-001"
title = "Reuse niucode's TUI through a library and hooks"
status = "doing"
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
- Step 1 is done in niucode `aac20a1`: `niucode-cli` exports `run()`, `TurnHooks`, and `NoHooks`. The hooks are not yet invoked; wiring them and moving the renderer are the remaining steps.

## Acceptance

- [ ] A niuse binary runs niucode's TUI with memory injected and candidates confirmed.
- [ ] niuse no longer owns a renderer or a second terminal input loop.
- [ ] The hook trait is generic: no identity or memory types cross into niucode.
- [ ] Contract tests still pass against the pinned revision.

## Notes

This is the only large piece of duplication left. Keeping two repos is still the decision (ADR-0003); this card removes the reason the boundary felt wrong, without merging the products.
