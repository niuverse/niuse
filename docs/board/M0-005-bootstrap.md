+++
id = "M0-005"
title = "Cold start and the review queue"
status = "done"
milestone = "M0"
module = "bootstrap"
size = "M"
priority = "P1"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-004"]
docs = ["plan"]
+++

## Scope

- `niuse bootstrap [--limit N] [--since-days N]` scans `~/.niucode/sessions`.
- Only user / assistant text is sent; tool output is dropped.
- One tool-free extraction returns `{kind, text, tags}` and writes `inbox.md`.
- `niuse review` walks the queue with `y` / `n` / `s` / `q`; accepted entries become memory.

## Acceptance

- [x] Empty sessions are skipped; only sessions with dialogue cost a model call.
- [x] Candidates are deduplicated against existing memory and within the batch.
- [x] Inbox format has a round-trip test.
- [x] A real run mined a sandbox lesson that review accepted into memory.

## Notes

Default cap is 40 sessions. Cost is roughly 2k input tokens per session, one-off.
