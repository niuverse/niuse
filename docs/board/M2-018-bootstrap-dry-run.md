+++
id = "M2-018"
title = "Bootstrap dry run"
status = "done"
milestone = "M2"
module = "bootstrap"
size = "S"
priority = "P2"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-005"]
docs = ["plan"]
+++

## Scope

- `niuse bootstrap --dry-run` selects sessions exactly as a real run would, then stops.
- It prints each session id and the transcript size in characters.
- No provider request is made, so the preview is free.
- Empty or tool-only sessions are still filtered before the preview.

## Acceptance

- [x] Dry run lists the selected sessions and their transcript sizes.
- [x] No model call and no inbox write.
- [x] The selection path is shared with the real run.
- [x] The CLI path was exercised with a real session directory.

## Notes

Bootstrap can cost a model call per session; a free preview is the difference between "try it" and "avoid it".
