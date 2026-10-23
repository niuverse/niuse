+++
id = "M10-021"
title = "Web computer entry and status"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-019"]
+++

## Problem

The product web surface could not start computer use and gave no indication
whether a browser prompt would have desktop access.

## Scope

- Forward niuse's global `--computer` to `niucode::web::Options`.
- Warn at startup when a configured model has no image capability.
- Show the kernel-provided computer policy/armed chip in the web header.
- Keep `off` the default and the runtime absent.
- Browser approvals remain the gate for `control` input.

## Acceptance

- [x] `niuse web --computer observe` forwards the policy.
- [x] `/api/status` and the header chip expose policy/armed.
- [x] Text-only models warn before a screenshot.
- [x] Off stays zero-backend.
- [x] Web feature compiles/clippy passes.
- [x] No browser arm/stop is claimed.

## Notes

Arm/stop is provided by kernel M9-106 (`POST /api/computer`); the product only
selects and reports policy.
