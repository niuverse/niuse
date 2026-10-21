+++
id = "M2-057"
title = "Relative --thread paths resolve consistently"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-015"]
docs = ["architecture"]
+++

## Scope

- `--thread` is treated as a path argument: relative paths resolve against the process working directory, absolute paths are kept.
- The same helper feeds `App`, `threads`, and `status`, so the active marker and status agree.
- With no `--thread`, the store's session path is used exactly as before.
- The absolute/default cases have a unit test.

## Acceptance

- [x] A relative thread path can no longer miss the active marker because one copy stayed relative.
- [x] `--thread` with an absolute path is unchanged.
- [x] The default path is unchanged.
- [x] One resolution point instead of three inline copies.
- [x] End-to-end: from the home directory, `--thread thread.jsonl` reports `active: true`.

## Notes

Three call sites each did `unwrap_or_else(session_path)`; the path was never normalized. The bug only showed when a relative path met the absolute listing.
