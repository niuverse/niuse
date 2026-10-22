+++
id = "M3-002"
title = "Distribution claims match the repository"
status = "done"
milestone = "M3"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M3-001"]
docs = ["distribution"]
+++

## Problem

`docs/distribution.md` claimed `cargo install --git https://github.com/niuverse/niuse` worked. A clean clone has no sibling niucode, and the `path = "../niucode/..."` dependencies cannot be fetched, so the command fails before building.

## Evidence

- Cloned the repository into `/tmp` and ran `cargo metadata`: `failed to load manifest for dependency niucode-agent`.
- `cargo install --path crates/niuse-cli --root /tmp/niuse-install --locked --debug` from the two-checkout workspace installed a working `niuse 0.1.0`.

## Fix

- Document the two-checkout build and `cargo install --path` as what works.
- Add a section proving why `cargo install --git` cannot work.
- Replace the misleading options-table row with "keep the two-checkout build" and note that converting to git dependencies is a separate change.

## Acceptance

- [x] No document claims `cargo install --git` works.
- [x] The clean-clone failure is quoted with the real error.
- [x] The working install path is verified and recorded.
- [x] crates.io blocking reason is unchanged and still accurate.

## Notes

A distribution note that recommends a command which cannot work is worse than saying "not yet": it burns the reader's first five minutes and hides the real blocker.
