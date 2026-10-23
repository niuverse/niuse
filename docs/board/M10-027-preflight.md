+++
id = "M10-027"
title = "Pre-publication preflight evidence"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-24"
completed = "2026-10-24"
depends_on = ["M10-022"]
+++

## Problem

The release preflight stopped at package verification because the registry
lacks the unpublished 0.1.8 crates. That hid whether the rest of the release
gates passed.

## Scope

- Add `--skip-package` to `release-preflight.sh` for the pre-publication window.
- Run it on clean trees and record the result.
- Keep the full preflight as the post-publish gate.
- No publication is claimed.

## Acceptance

- [x] Script help/syntax is valid.
- [x] Pre-publication preflight completes.
- [x] niucode tests/clippy/dashboard and niuse gates pass.
- [x] Package verification is explicitly skipped, not silently ignored.
- [x] Evidence is recorded.
- [x] Checks pass.

## Notes

The full preflight remains mandatory after publishing niucode 0.1.8 and before
publishing niuse 0.1.14.
