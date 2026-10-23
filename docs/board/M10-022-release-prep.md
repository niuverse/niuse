+++
id = "M10-022"
title = "Release preparation for niucode 0.1.8 / niuse 0.1.14"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-021"]
+++

## Problem

The new computer/multimodal crates and product flags need a versioned release,
but publishing from a dirty or unverified tree is unsafe.

## Scope

- Bump niucode to `0.1.8` (all sibling requirements) and niuse to `0.1.14`.
- Pin the 0.1.8 kernel revision.
- Add release notes and a dry-run evidence file.
- Run `publish-niucode.sh --check`, `publish-niuse.sh --check`, workspace
  tests/clippy, dashboard, and docs checks.
- Record the expected pre-publication package blocker.
- Do not publish without maintainer approval.

## Acceptance

- [x] Versions and sibling requirements bumped.
- [x] `niucode.rev` points at the 0.1.8 prep commit.
- [x] Release notes exist for both repositories.
- [x] Dry-run checks pass where publication is not required.
- [x] Pre-publication package blocker is documented.
- [x] Tests/clippy/docs pass.
- [ ] Publish to crates.io (human approval).

## Notes

`niucode-computer` packages successfully; platform crates cannot resolve their
unpublished `0.1.8` dependency yet. This is publish order, not a source bug.
