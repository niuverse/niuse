+++
id = "M2-003"
title = "Memory recall across expired entries"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-004"]
docs = ["architecture", "plan"]
+++

## Scope

- `niuse mem search <query> [--fresh]` searches every saved memory, including expired ones.
- `/mem recall <query>` does the same inside the REPL.
- Ranking prefers substring and tag matches over character-bigram similarity, which is weak for short queries.
- Results show age, expiry state, tags, and the entry id.

## Acceptance

- [x] Expired entries are searchable but still excluded from injection.
- [x] `--fresh` hides expired entries.
- [x] Substring and tag matches outrank unrelated entries; threshold is 0.2.
- [x] Core scoring has unit tests; CLI and REPL paths were exercised manually.

## Notes

Recall is the reason expired memory is kept instead of deleted: it leaves the prompt, not the store.
