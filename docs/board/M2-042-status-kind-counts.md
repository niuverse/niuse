+++
id = "M2-042"
title = "Kind counts in status"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-005", "M2-041"]
docs = ["architecture"]
+++

## Scope

- `niuse status` shows preference and lesson counts next to the memory total.
- `status --json` exposes `memory.preferences` and `memory.lessons`.
- The CLI integration test asserts both zero-state fields.
- `mem list` and `status` now describe the store the same way.

## Acceptance

- [x] Text output reads `6 条（preference 3 · lesson 3；…）`.
- [x] JSON parses with the new fields.
- [x] Empty stores report zeros, not missing keys.
- [x] No storage or prompt change.

## Notes

Two commands answering "what is in memory" should not disagree about the shape of the answer.
