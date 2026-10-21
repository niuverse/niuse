+++
id = "M2-052"
title = "Edit a memory during review"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-043"]
docs = ["architecture"]
+++

## Scope

- The review prompt gains `[e]`; it opens `$EDITOR` with the current text.
- Saving a changed, non-empty, secret-free text updates it and refreshes the confirmation date.
- Empty, unchanged, or secret-bearing edits are refused without touching the file.
- The action records `mem_review_edit`; `stats` reports kept/edited/deleted on a dedicated review line.

## Acceptance

- [x] An edit confirms the entry and records a metric.
- [x] Secret patterns are refused at review time too.
- [x] A vanished entry is reported, not panicked.
- [x] Stats text and JSON carry the edited count.

## Notes

Review used to force a binary keep/delete choice on stale text. Editing is the third option that does not lose the fact but fixes the wording.
