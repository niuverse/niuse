+++
id = "M2-065"
title = "Doctor checks loops and journal too"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-064"]
docs = ["architecture"]
+++

## Problem

The memory parse check stopped at memory. `loops.md` and `journal/*.md` are hand-editable too; a malformed promise line or an invalid date heading silently disappears from prompts and listings.

## Fix

- `Store::content_issues()` extends `memory_issues()` with:
  - loops: `- [` lines versus parsed promises, plus stray non-comment lines;
  - journal: `## ` headings and `- ` bullets versus parsed days/bullets per file.
- `doctor` reports one `parse` check for all three stores.
- The unit test plants a malformed loop and an invalid journal heading and asserts both are named.
- No automatic repair; files stay human-owned.

## Acceptance

- [x] A malformed promise line is reported as `loops.md`.
- [x] An invalid date heading and its bullet are reported for the journal file.
- [x] Clean stores report `all stored content parsed`.
- [x] The check is read-only.

## Notes

Same principle as M2-064: Markdown is editable, so malformed content must be visible, not silently dropped.
