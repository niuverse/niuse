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
- **Approvals**: computer approvals carry a bounded in-memory target thumbnail
  in the web card (never persisted); emergency stop clears it.
- **Privacy**: sensitive-window observation asks before returning an image;
  typed computer text is persisted only as length + SHA-256 by default, with
  an explicit debug `--computer-log-input`; screenshots stay ephemeral unless
  `--computer-keep-frames` is explicit; approvals show coordinates/keys/typed
  length, never typed text.
- **Doctor**: `niuse doctor` reports X11 readiness/XTEST or macOS TCC and
  points at the kernel `niucode computer doctor`.
- **Binding**: input rechecks window id/app/class/kind + geometry, not just
  id, before dispatch; typed text/key/scroll have hard per-action caps.
- **Red lines**: Super, Alt+Tab/Escape/F4, Ctrl+Alt+Delete/Backspace/F1-F12
  require approval in autonomous mode.
- **TUI badge**: policy + armed/paused is visible in the footer; stop updates it.
- **Audit**: a structured `computer_action` event records the captured/target
  window, observation id, purpose, result, and typed length + SHA-256, never
  text or images.
- **Vision budget**: every provider request carries at most four images; the
  oldest become a text marker.
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
grants. Run `niucode computer doctor [--json]` to check DISPLAY/TCC, XTEST,
window enumeration, and capture before enabling a policy. Real-desktop/macOS
acceptance steps are in the kernel's `docs/computer-acceptance.md` and
`scripts/computer-dogfood.sh`.

## Verification

```sh
./scripts/publish-niuse.sh --check
./scripts/release-preflight.sh
```

The kernel CI runs a real Xvfb observe → image → approved click → ButtonPress
turn. `--check` reports version/license/token/order. Preflight is expected to fail
until `niucode 0.1.8` is published, because its package step resolves the new
crates from the registry.

## Runbook

The kernel `docs/release-runbook.md` records the publish order, index waits,
clean-install checks, tag/evidence steps, rollback, and the explicit human gate.

## Blockers

- Human approval to publish.
- Publish niucode 0.1.8 first.
- Then verify a clean install before/after uploading niuse.
