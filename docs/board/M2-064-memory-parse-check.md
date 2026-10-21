+++
id = "M2-064"
title = "Doctor flags malformed memory sections"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-005"]
docs = ["architecture"]
+++

## Problem

Memory files are hand-editable Markdown. A typo in the required fields makes a `### m-…` section silently unparseable: the entry disappears from prompts and listings with no signal.

## Fix

- `Store::memory_issues()` compares `### m-…` headers with parsed entries per file.
- `doctor` reports `all memory sections parsed`, a warning naming the file and the unparsed section count, a warning when a file has content but no parseable entries, or stray non-field lines among valid entries.
- No automatic repair; the file stays the user's.

## Acceptance

- [x] A store with one valid and one broken section reports one issue.
- [x] A file containing only garbage reports `has content but no parseable entries`.
- [x] Stray prose among valid entries is reported.
- [x] A clean store reports none.
- [x] The doctor warning names `preferences.md`.
- [x] The check is read-only.

## Notes

Silent data loss is worse than a noisy warning here: the whole point of Markdown storage is that a human can edit it.
