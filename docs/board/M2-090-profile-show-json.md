+++
id = "M2-090"
title = "Profile files as JSON"
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

- `niuse user show --json` and `niuse identity show --json` emit `{path, text}`.
- A missing file is `text: null`, not an empty string; the path is always present.
- Text output is unchanged.
- An end-to-end test checks both commands on an empty home.

## Acceptance

- [x] Both objects parse and carry an absolute path.
- [x] Missing text is null.
- [x] Text views are unchanged.
- [x] No storage change.

## Notes

The two human-authored layers now have the same machine-readable shape as memory and journal.
