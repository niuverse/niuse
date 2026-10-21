+++
id = "M2-007"
title = "Memory maintenance: show and edit"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-001"]
docs = ["architecture"]
+++

## Scope

- `niuse mem show <id>` prints kind, tags, created, confirmed, source, and text.
- `niuse mem edit <id>` opens the text in `$EDITOR`, normalises it to one line, rescans for secrets, and refreshes the confirmation date.
- `/mem show <id>` works in the REPL; candidate editing reuses the same editor helper.
- IDs accept `m-`, `l-`, or no prefix.

## Acceptance

- [x] Show prints every field a reviewer needs.
- [x] Edit updates the text and confirmed date, or reports no change.
- [x] A secret in the edited text is refused and the stored entry is untouched.
- [x] The editor helper is shared with candidate editing; ID prefixes have a test.

## Notes

Correcting a wrong memory is as important as adding one: a bad entry is injected on every turn until it is fixed.
