+++
id = "M2-002"
title = "Show a full unified diff at approval"
status = "done"
milestone = "M2"
module = "repl"
size = "S"
priority = "P2"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-003"]
docs = ["architecture"]
+++

## Scope

- Render `ChangePreview` as unified hunks: file header, `@@` ranges, context, additions, removals.
- Reuse niucode's `Toolbox::preview` result; niuse copies no path policy and performs no extra reads.
- Cap the rendered diff at 80 lines with an explicit "more line(s)" marker.
- Keep the approval prompt below the diff; non-TTY still falls back to a line reply.

## Acceptance

- [x] Write / edit approval shows real hunks with `+` and `-` lines.
- [x] A new file is marked as new; command and note previews still render.
- [x] Long previews are capped instead of flooding the terminal.
- [x] Rendering has unit tests; the security boundary is unchanged.

## Notes

This was the last known gap between niuse's approval prompt and niucode's TUI preview.
