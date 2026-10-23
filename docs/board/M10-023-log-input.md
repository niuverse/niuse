+++
id = "M10-023"
title = "Product opt-in for full typed desktop text"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-24"
completed = "2026-10-24"
depends_on = ["M10-019"]
+++

## Problem

niuse exposed computer policy but not the kernel's explicit typed-text logging
opt-in, so a product user could not choose the documented debug behavior.

## Scope

- Add global `--computer-log-input`.
- Pass it to REPL/exec agent configs and the TUI options.
- Warn at startup when it is enabled with a non-off policy.
- Keep the default redacted.
- Parse test.

## Acceptance

- [x] Flag reaches REPL/exec/TUI configs.
- [x] Default remains length + SHA-256.
- [x] Warning is explicit.
- [x] No web opt-in is added.
- [x] Tests/checks pass.

## Notes

Debug-only; full desktop text is sensitive. The product does not weaken the
kernel default.
