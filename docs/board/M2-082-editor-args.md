+++
id = "M2-082"
title = "Editor command respects arguments and VISUAL"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-003"]
docs = ["architecture"]
+++

## Problem

`$EDITOR` was passed whole to `Command::new`, so a common value like `code --wait` was treated as one executable name and failed. An empty `$EDITOR` failed the same way, and `$VISUAL` was ignored.

## Fix

- `$VISUAL` wins, then `$EDITOR`, then `vi`.
- The value is split into program plus arguments; the temp path is appended as the final argument.
- Empty values fall back to `vi`.
- A pure `editor_parts` test covers `code --wait`, empty, and a bare program.

## Acceptance

- [x] `code --wait` becomes program plus one argument.
- [x] Empty input yields `vi`.
- [x] The edited file path is still the last argument.
- [x] Temporary files stay private.

## Notes

This does not attempt shell quoting; editors with quoted paths should be wrapped in a small script. Ordinary flags are the common case.
