+++
id = "release-014"
title = "Release 0.1.14 (draft)"
kind = "doc"
status = "draft"
order = 33
summary = "Draft notes: product-wide computer policy, vision warnings, web status, and cross-session memory evidence on niucode 0.1.8."
+++

## Status

Prepared 2026-10-23. `niuse-core` and `niuse` are bumped to `0.1.14`,
requiring `niucode 0.1.8`. Publication is pending maintainer approval; nothing
has been uploaded for this version.

## Why 0.1.14

`niuse` now exposes native computer use on the REPL, exec, TUI, and web surfaces,
and needs the 0.1.8 kernel crates (`niucode-computer`, platform backends, the
vision capability probe, and web computer status).

## Highlights

- **Computer policy everywhere**: global `--computer
  off|observe|control|autonomous` for REPL/exec/TUI/web.
- **REPL/TUI controls**: `/computer`, `/computer arm`, `/computer stop` with
  emergency disarm and input release.
- **Vision warning**: text-only models warn before a screenshot is requested.
- **Web**: policy chip, `/api/status` computer state, and token-gated
  arm/stop; browser approvals remain the input gate.
- **Memory evidence**: reopened attachment store and reopened Markdown memory
  both feed a new session.
- **Boundary**: platform backends live in niucode crates; the default niuse
  binary still does not link niucode's TUI.

## Install shape after release

```sh
cargo install niuse --version 0.1.14 --locked
cargo install niuse --version 0.1.14 --features web --locked
cargo install niuse --version 0.1.14 --features tui --locked
```

Computer use additionally requires a vision-capable model and, on macOS, TCC
grants.

## Verification

```sh
./scripts/publish-niuse.sh --check
./scripts/release-preflight.sh
```

The kernel CI runs a real Xvfb observe → image → approved click → ButtonPress
turn. `--check` reports version/license/token/order. Preflight is expected to fail
until `niucode 0.1.8` is published, because its package step resolves the new
crates from the registry.

## Blockers

- Human approval to publish.
- Publish niucode 0.1.8 first.
- Then verify a clean install before/after uploading niuse.
