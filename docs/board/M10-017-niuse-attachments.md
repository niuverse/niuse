+++
id = "M10-017"
title = "niuse exec attachments"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P0"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-016"]
docs = ["roadmap"]
+++

## Problem

niucode could attach images/scripts, but niuse's own one-shot command had no
way to pass them; users had to drop to the kernel CLI.

## Scope

- Add repeatable `--attach`/`--image` to `niuse exec`.
- Reuse `niucode-attachments` and `driver::run_turn_with_attachments`.
- Store attachments under the niuse home and enforce flag kinds.
- Keep the text-only `run_turn` path and no niucode fork.
- Tests for parsing and ingestion.

## Acceptance

- [x] `niuse exec --attach --image` parses.
- [x] Text flag rejects images and vice versa.
- [x] The attached turn goes through niucode's budgeted attachment runtime.
- [x] No provider/backend logic is duplicated in niuse.
- [x] `check.sh` passes.

## Notes

TUI/Web composer attachment UX is next. The default niuse binary still does not
link niucode's TUI/web feature.
