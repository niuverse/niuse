+++
id = "M2-075"
title = "Memory id completion accepts the m- prefix"
status = "done"
milestone = "M2"
module = "repl"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-034"]
docs = ["architecture"]
+++

## Problem

Listings show memory ids as `m-7f3a`, but Tab completion matched only the bare `7f3a`. Typing the visible form `/mem show m-7f` and pressing Tab produced no candidates.

## Fix

- The memory completer strips an optional `m-` before matching and puts it back in both display and replacement.
- Loop ids already contained `l-`; that path is unchanged.
- The id-completion test now covers both bare and prefixed fragments.

## Acceptance

- [x] `/mem show m-<prefix>` completes to `m-<id>`.
- [x] Bare prefixes still complete to the bare id.
- [x] Loop completion remains `l-<id>`.
- [x] Display and replacement agree.

## Notes

Completion should accept the spelling the product prints. This was a small mismatch between two surfaces of the same id.
