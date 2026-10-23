+++
id = "M10-010"
title = "niuse publish dry run"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P0"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-008"]
docs = ["roadmap"]
+++

## Problem

Only niucode packages had been dry-run. niuse's package manifest and file set
were unverified.

## Scope

- Dry-run `niuse-core` and `niuse`.
- Record pass/verify/upload.
- State that default features do not compile `web`/`tui`.
- No upload or version bump.

## Acceptance

- [x] Both crates package and verify.
- [x] Upload is aborted for both.
- [x] Evidence states the niucode 0.1.6 resolution.
- [x] Evidence states the feature gap.
- [x] Human release approval remains required.

## Notes

The next release must rerun this after pinning niucode 0.1.7 and exercise both
opt-in features.
