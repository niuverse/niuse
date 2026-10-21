+++
id = "M2-060"
title = "Export uses a readable title"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-045", "M2-012"]
docs = ["architecture"]
+++

## Scope

- Exported Markdown starts with the first user message (one line, 60 chars) instead of the fixed `# niuse thread`.
- The existing title derivation is reused, so threads, `threads --json`, and export agree.
- Empty or unreadable logs fall back to `niuse thread`.
- The metadata blockquote and body are unchanged.

## Acceptance

- [x] An exec log exports as `# 只回答两个字：pong`.
- [x] The fallback keeps old exports readable.
- [x] Protocol blocks and `0600` behavior are untouched.
- [x] No new state or dependency.

## Notes

An exported file is often the only artifact a human sees. Its first line should say what the session was about.
