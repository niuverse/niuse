+++
id = "M2-040"
title = "Colored approval diffs"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-002"]
docs = ["architecture"]
+++

## Scope

- Approval summaries are bold; warnings are yellow.
- Diff headers are bold, additions green, removals red, hunk headers cyan.
- Styling applies only on a TTY and is suppressed by `NO_COLOR` or a pipe.
- The style classifier is a pure function with a unit test.

## Acceptance

- [x] `+`, `-`, `@@`, and file headers map to distinct styles.
- [x] Context lines are unchanged.
- [x] The prompt and the 80-line cap are untouched.
- [x] No new dependency.

## Notes

Color is the cheapest way to make a long diff scannable before a y/N decision. The decision itself is still one line.
