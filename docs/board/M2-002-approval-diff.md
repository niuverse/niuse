+++
id = "M2-002"
title = "Show a full unified diff at approval"
status = "backlog"
milestone = "M2"
module = "repl"
size = "S"
priority = "P2"
updated = "2026-10-21"
depends_on = ["M0-003"]
docs = ["architecture"]
+++

## Scope

- Show the structured `ChangePreview` diff instead of a summary plus +/- counts.
- Reuse niucode's `Toolbox::preview` result; do not copy path policy into niuse.
- Page or fold long diffs so the terminal stays readable.

## Acceptance

- [ ] Write / edit approval shows real hunks.
- [ ] A rejection produces events consistent with behaviour.
- [ ] Non-TTY falls back to line-by-line confirmation.

## Notes

The security boundary is unchanged; this only surfaces information that already exists.
