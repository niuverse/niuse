+++
id = "M2-021"
title = "User profile editing"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-001"]
docs = ["architecture"]
+++

## Scope

- `niuse user show|edit` and `/user` / `/user edit` expose `user.md`.
- Editing uses the multiline `$EDITOR` helper and keeps the file `0600`.
- A missing file opens a small template with the three questions that matter: current work, working style, red lines.
- The edit is recorded in metrics.

## Acceptance

- [x] Show prints the path and content, or a hint when missing.
- [x] Edit writes the file through a scripted editor and `user show` reflects it.
- [x] Empty edits report no change.
- [x] Permissions remain `0600`.

## Notes

`identity.md` says who it is; `user.md` says who you are. The second one had no manual path, only bootstrap.
