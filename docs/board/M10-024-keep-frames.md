+++
id = "M10-024"
title = "Product opt-in for kept observation frames"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-24"
completed = "2026-10-24"
depends_on = ["M10-023"]
+++

## Problem

The kernel gained an explicit screenshot Keep frame, but the product could not
select it and offered no warning.

## Scope

- Add global `--computer-keep-frames`.
- Pass it to REPL/exec/TUI agent configs and TUI options.
- Warn when enabled with a non-off policy.
- Keep the default ephemeral.
- Parse test.

## Acceptance

- [x] Flag reaches all interactive/one-shot configs.
- [x] Default writes no screenshot.
- [x] Warning is explicit.
- [x] No web retention path is added.
- [x] Tests/checks pass.

## Notes

Debug/audit-only. Screenshots can contain secrets; retention cleanup is manual.
