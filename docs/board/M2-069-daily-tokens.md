+++
id = "M2-069"
title = "Daily token trend"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-066"]
docs = ["architecture"]
+++

## Scope

- Per-day summaries now accumulate input/output/cached tokens.
- `stats --trend` prints `12k in · 400 out` beside turns and acceptance.
- `stats --json` day records carry the three token fields.
- Token detail parsing is one helper shared by the total and daily paths.

## Acceptance

- [x] A day with a token record reports its counts.
- [x] The trend line includes compact input/output numbers.
- [x] JSON day entries gain the fields without breaking existing keys.
- [x] The daily-summary test covers tokens.

## Notes

Totals answer "what did this week cost"; the trend answers "is it getting cheaper or noisier".
