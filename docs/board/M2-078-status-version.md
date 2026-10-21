+++
id = "M2-078"
title = "Status reports the version"
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

- `niuse status` begins with `version  0.1.0`.
- `status --json` carries the same string as `version`.
- The CLI test asserts a non-empty version field.
- The value comes from `CARGO_PKG_VERSION`; no second constant.

## Acceptance

- [x] Text and JSON report the package version.
- [x] The value is not hardcoded in the test.
- [x] No prompt or storage change.
- [x] Works on an empty home.

## Notes

Bug reports need the binary version and the pinned niucode revision; doctor already shows the pin.
