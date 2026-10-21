+++
id = "M2-023"
title = "Secret scan for identity and profile edits"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-004"]
docs = ["architecture"]
+++

## Scope

- `identity edit`, `user edit`, and `identity init` scan the final text before writing.
- `bootstrap --profile` refuses a generated draft that contains a secret pattern.
- A refusal leaves the existing file untouched and exits non-zero.
- Memory, journal, and loop writes already had the same guard.

## Acceptance

- [x] A scripted editor inserting `sk-...` is refused for identity and user.
- [x] The existing identity file is unchanged after the refusal.
- [x] The profile draft path checks before `write_user`.
- [x] The scan uses the same built-in patterns as other writes.

## Notes

Identity and user text is injected on every turn; a leaked key there would be sent to the provider continuously. The guard belongs at the write, not only in memory.
