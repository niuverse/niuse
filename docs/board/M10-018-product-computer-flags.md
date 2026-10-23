+++
id = "M10-018"
title = "Product computer flags for exec and TUI"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-017"]
+++

## Problem

The kernel could run computer use, but niuse had no product entry: `niuse exec`
and `niuse tui` did not accept a policy, so users could not enable observation
or control from the product.

## Scope

- Add `--computer off|observe|control|autonomous` to `niuse exec` and
  `niuse tui`.
- `exec` builds the platform runtime through `driver::configure_computer`
  (X11 on Linux, CoreGraphics/CGEvent on macOS), then arms it.
- Missing display/TCC permission warns and leaves computer off; invalid policy
  is a CLI error.
- TUI forwards the flag to niucode's shared TUI options.
- Parse test and README/help coverage.

## Acceptance

- [x] `niuse exec --computer observe` parses and wires the config.
- [x] TUI forwards the policy only when the tui feature is enabled.
- [x] Platform backends stay in niucode crates, not a fork.
- [x] Off remains the default and creates no backend.
- [x] Existing attachment/MCP behavior is unchanged.
- [x] Tests/checks pass.

## Notes

The web surface still needs a policy switch; the REPL switch is M10-019. The default niuse binary
now links the platform computer crates, but not niucode's TUI.
