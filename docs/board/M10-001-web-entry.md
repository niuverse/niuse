+++
id = "M10-001"
title = "niuse web entry"
status = "done"
milestone = "M10"
module = "cli"
size = "M"
priority = "P0"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M8-008"]
docs = ["roadmap"]
+++

## Problem

The web surface existed only as `niucode web`. It used the default coding-agent
prompt, not niuse's identity, user profile, memory, goals, and plan.

## Scope

- Add `niuse web [--port]`, loopback only.
- Compose the product prompt from the seven stores.
- Delegate transport/chat/approvals/Markdown to `niucode::web`; do not fork it.
- Use the niuse thread (`thread.jsonl`) as the session log.
- Keep the existing provider/model/cwd settings.

## Acceptance

- [x] `niuse web --help` exists and defaults to port 8788.
- [x] The system prompt is composed from identity/user/goals/plan/memory/loops/journal.
- [x] The server uses niucode's web route, SSE, approval, and rendering code.
- [x] The session is the niuse thread, not a second store.
- [x] No remote assets or non-loopback binding.
- [x] Build and clippy pass.
- [x] `niucode` stays optional behind the `web` feature.

## Notes

This is the web entry, not the full product shell. Identity/memory/plan sidebars
and product-specific routes are the next M10 slices. `web` is an opt-in feature
so the default binary still does not link niucode.
