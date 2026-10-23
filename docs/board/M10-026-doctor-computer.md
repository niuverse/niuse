+++
id = "M10-026"
title = "Computer availability in niuse doctor"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-24"
completed = "2026-10-24"
depends_on = ["M10-025"]
+++

## Problem

`niuse --computer` could warn that desktop use was unavailable, but `niuse
doctor` did not show a computer check, so users had to know about the kernel
doctor command.

## Scope

- Add a `computer` line to `niuse doctor` (human and JSON).
- Linux: warn when DISPLAY is unset; otherwise try X11, count windows, report
  XTEST.
- macOS: report Screen Recording and Accessibility preflight.
- Other platforms: warn unsupported.
- Never fail the overall doctor for a missing optional desktop permission.
- Point to `niucode computer doctor` for the deep read-only probe.

## Acceptance

- [x] Headless Linux shows a warning with the DISPLAY remedy.
- [x] X11 ready shows window count and XTEST.
- [x] macOS shows both TCC permissions.
- [x] The check does not inject input or write attachments.
- [x] JSON/human output both include it.
- [x] Checks pass.

## Notes

The doctor is diagnostic only; it does not arm a policy or grant permissions.
