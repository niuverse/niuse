+++
id = "M10-025"
title = "Use the kernel computer doctor from niuse"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-24"
completed = "2026-10-24"
depends_on = ["M10-021"]
+++

## Problem

When `niuse --computer` warned that the desktop was unavailable, the user had
no product-level diagnostic path.

## Scope

- Document `niucode computer doctor [--json]` as the first check.
- The command reports DISPLAY/TCC, XTEST, windows, and capture without input.
- Keep the backend implementation in the kernel; niuse does not fork it.
- No new niuse command is added.

## Acceptance

- [x] The diagnostic is linked from the product release notes/status.
- [x] It is read-only.
- [x] Missing DISPLAY/TCC has an actionable message.
- [x] Checks pass.

## Notes

A product `niuse doctor` integration can wrap this later; the kernel command is
stable enough to use now.
