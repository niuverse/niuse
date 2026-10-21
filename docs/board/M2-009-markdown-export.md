+++
id = "M2-009"
title = "Markdown export"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-003"]
docs = ["architecture"]
+++

## Scope

- `niuse export [--out FILE] [--session PATH]` projects a session log to Markdown.
- User and assistant turns keep their order; tool calls and results are fenced.
- Protocol blocks are stripped, exactly as the REPL strips them.
- Tool output is clipped at 4,000 characters; exported files are `0600`.

## Acceptance

- [x] Turns render in order with a metadata header.
- [x] Protocol blocks never leak into the export.
- [x] Long tool output is clipped with a marker.
- [x] Writes to `--out` are private; stdout works without a file.
- [x] Unit tests cover ordering, clipping, and block stripping.

## Notes

The event log remains the source of truth. The export is a read-only projection for review, backup, or attaching a transcript to an issue.
