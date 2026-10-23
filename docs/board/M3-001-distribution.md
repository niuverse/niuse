+++
id = "M3-001"
title = "Release and dependency strategy"
status = "backlog"
milestone = "M3"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-21"
depends_on = ["M0-003"]
docs = ["plan", "distribution"]
+++

## Scope

- Decide the niucode dependency: publish niucode crates first, convert to pinned git revisions, or keep the two-checkout build.
- Add `license`, `repository`, `description`, and other release metadata.
- Make `cargo package` / `cargo publish --dry-run` reproducible.
- Confirm crate-name availability; rename the crate if needed while keeping the `niuse` binary.

## Acceptance

- [ ] `cargo package -p niuse` passes (currently blocked by path dependencies).
- [ ] The install path is reproducible on a clean machine.
- [ ] License and naming strategy are documented in the README.

## Notes

Findings and options are recorded in [distribution](../distribution.md).
2026-10-22: the original crates are published, but `niucode-mcp` is not on
crates.io, so `cargo package -p niuse` fails on that dependency. `niucode-mcp`
now packages and passes `cargo publish --dry-run`; the publish order includes
it. This card stays backlog until the authorized upload and a passing
`cargo package -p niuse`.

2026-10-23: `niucode-mcp` 0.1.5 is published. The next blocker is the niuse
version: crates.io core 0.1.11 predates the plan API. M3-006 prepares 0.1.12;
core must be uploaded before the CLI package can verify.

Current failure: `all dependencies must have a version requirement specified when packaging`, pointing at the `niucode-agent` path dependency.
