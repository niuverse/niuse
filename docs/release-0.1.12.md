+++
id = "release-012"
title = "Release 0.1.12"
kind = "doc"
status = "living"
order = 31
summary = "Published 2026-10-23: niuse-core 0.1.12 and niuse 0.1.12 are on crates.io; clean-root install verified."
+++

## Status

Published · 2026-10-23. `niuse-core` 0.1.12 and `niuse` 0.1.12 are on
crates.io, and a clean-root install reports `niuse 0.1.12`. This page records
the preflight and the exact order; it is history, not a pending action.

## Why 0.1.12

`niuse-core` 0.1.11 on crates.io predates `PlanStep` and
`compose_with_plan`, so `cargo package -p niuse` fails when it resolves the
registry core. The workspace is bumped to 0.1.12. `niucode-mcp` 0.1.5 is
already published.

## Release result

- `cargo search niuse` → 0.1.12
- `cargo install niuse --version 0.1.12 --locked --root /tmp/niuse-0.1.12`
  installed the binary; `niuse --version` → `niuse 0.1.12`
- The first CLI upload failed verification: pack scripts sat outside the crate
  and registry niucode 0.1.5 predated `EmbedderTool`. The fix embeds the packs
  under `crates/niuse-cli/assets` and ships niucode 0.1.6.
- `niuse-core` 0.1.12 was already uploaded on the first attempt; the retry
  skipped it and published only the CLI.

## Preflight (verified read-only, before the first upload)

- `./scripts/check.sh` passes.
- `cargo package -p niuse-core` passes.
- `cargo publish -p niuse-core --dry-run` reaches the upload step and aborts
  at the dry-run warning.
- `./scripts/publish-niuse.sh --check` reports version 0.1.12, the license, and
  a credentials file.
- `./scripts/dogfood-trend.sh` returns `insufficient data` (2 active days), not
  a regression; the publish script treats that as non-blocking.

## Commands

Requires explicit human authorization and a clean tree:

```sh
cd /home/galbot/Projects/niuverse/niuse
./scripts/publish-niuse.sh --execute
```

The script:

1. refuses a dirty tree or a missing token;
2. runs the dogfood trend and refuses a regression;
3. publishes `niuse-core`, waits 20s for the sparse index;
4. publishes `niuse`;
5. skips a crate whose version is already on the index, so a partial run can
   be retried.

## Verify after upload

```sh
cargo search niuse-core --limit 1
cargo search niuse --limit 1
cargo package -p niuse
cargo install niuse --version 0.1.12 --locked --root /tmp/niuse-0.1.12
/tmp/niuse-0.1.12/bin/niuse --version
```

Then update `M3-006`, `M3-001`, [distribution](distribution.md), and
[status](status.md) from “prepared” to the published version.

## If it fails

- Do not reuse 0.1.12 after a failed upload; bump to 0.1.13.
- A partial run is safe to retry: already-published crates are skipped.
- A bad release is yanked, not deleted: `cargo yank --version 0.1.12 <crate>`,
  then ship the next patch.
