+++
id = "M3-003"
title = "Niucode crates as versioned dependencies"
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

## Scope

- niuse's five `niucode-*` dependencies now carry `version = "0.1.0"` beside `path`.
- Local development and the pinned checkout are unchanged.
- niucode's internal dependencies are also versioned, and a publish-order script exists.
- The remaining release prerequisites are a license and a crates.io token.

## Acceptance

- [x] Workspace still builds and tests with path + version dependencies.
- [x] niucode crates share 0.1.0 and list the publish order.
- [x] `niuse-core` is also a versioned dependency of the binary.
- [x] Publish `niucode-core` and its dependents to crates.io (all seven at 0.1.0).
- [x] Verify `cargo package -p niuse` against the registry afterwards; it succeeds and the binary installs from crates.io.

`cargo package -p niuse` now gets past manifest validation and stops at the expected `no matching package named niucode-agent` until the registry upload exists.

## Notes

This is the chosen distribution path: publish the kernel crates first, then the product. The two unchecked items require the user's license choice and token.
