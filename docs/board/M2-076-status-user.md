+++
id = "M2-076"
title = "Status shows whether a user profile exists"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-022"]
docs = ["architecture"]
+++

## Scope

- `niuse status` prints a `user` line next to `identity`, with `已设置` / `未设置`.
- `status --json` already carried the boolean; this aligns the text view.
- The CLI integration test asserts the line exists.

## Acceptance

- [x] The text view reports profile presence.
- [x] JSON is unchanged.
- [x] Empty and populated homes both work.
- [x] No prompt or storage change.

## Notes

Identity and user are the two human-authored layers; showing only one of them made the status page look like the profile did not exist.
