+++
id = "M10-011"
title = "Release feature dry run"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P0"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-010"]
docs = ["roadmap"]
+++

## Problem

Default-feature packaging passed, but the opt-in features were not exercised.
The `web` feature depends on niucode APIs that are newer than the registry.

## Scope

- Dry-run `niuse --features tui` and `--features web`.
- Record the exact result and error.
- Do not change code or versions.
- Add the release gate: publish niucode 0.1.7 first.

## Acceptance

- [x] `tui` packages and verifies.
- [x] `web` fails only at `niucode::web` against 0.1.6.
- [x] Evidence records the exact error.
- [x] The current sibling checkout passes `--features web`.
- [x] No upload or version bump.

## Notes

This is an expected cross-repo dependency, not a regression. 0.1.13 cannot be
published with `web` until 0.1.7 is live.
