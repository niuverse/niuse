+++
id = "M2-029"
title = "Doctor JSON"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-005"]
docs = ["architecture"]
+++

## Scope

- `niuse doctor --json` emits `{checks, failures, warnings}`.
- Each check has `level` (`ok` / `warn` / `fail`), `label`, and `detail`.
- Human output is unchanged when the flag is absent.
- Exit semantics stay the same: failures are non-zero, warnings are zero.

## Acceptance

- [x] JSON parses and contains every check.
- [x] Levels serialize in lowercase.
- [x] The report counters still have a unit test, now also covering serialization.
- [x] Existing human output is untouched.

## Notes

CI and setup scripts can now assert on the same facts a human sees, instead of scraping `[ok]` prefixes.
