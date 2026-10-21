+++
id = "M2-032"
title = "Per-turn usage and transcript styling"
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

- Each turn ends with a compact usage line: input, output, and cached tokens when present.
- Tool calls and successful tool results are dim; failures are red; warnings are yellow.
- Styling is enabled only on a TTY and is suppressed by `NO_COLOR` or a pipe.
- Token counts use `999`, `1.5k`, `12k`; the formatter has a unit test.

## Acceptance

- [x] A real turn printed `1.9k in · 2 out`.
- [x] Piped output stays plain, so integration tests and exports are unaffected.
- [x] Formatting is covered by a unit test.
- [x] No new dependency for color.

## Notes

Cost awareness should be one dim line, not a dashboard. The transcript remains readable when piped or when colors are disabled.
