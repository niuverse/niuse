+++
id = "M2-049"
title = "Full diff on demand at approval"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-002", "M2-040"]
docs = ["architecture"]
+++

## Scope

- The approval prompt still shows at most 80 diff lines by default.
- When the preview was capped, `d` prints the complete diff and asks again; `y` accepts, anything else denies.
- If nothing was capped, `d` is not offered and is treated as a denial.
- Parsing of `y/yes/d/diff` is a pure function with a unit test.

## Acceptance

- [x] `d` only appears when the preview is truncated.
- [x] `d` reprints the full preview without deciding.
- [x] Denial remains the default for unknown input.
- [x] The 80-line default cap is unchanged.

## Notes

Reviewing a large edit from a terminal should be possible without approving it first. The extra key keeps the common case short.
