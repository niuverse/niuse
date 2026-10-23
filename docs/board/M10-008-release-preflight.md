+++
id = "M10-008"
title = "Cross-repository release preflight"
status = "done"
milestone = "M10"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-005"]
docs = ["roadmap"]
+++

## Problem

Release checks were a list of commands in prose. It was easy to verify niuse but
forget niucode's MSRV/package/dashboard, or to release from a dirty tree.

## Scope

- Add `scripts/release-preflight.sh`.
- Require both repos clean.
- Verify `niucode.rev` equals the sibling checkout HEAD.
- Run niucode MSRV, tests, clippy, package, dashboard check.
- Run niuse MSRV (default/web/tui) and `scripts/check.sh`, then drift.
- Explicitly does not publish anything.
- Document it in the 0.1.13 draft.

## Acceptance

- [x] Script exits on a dirty tree.
- [x] Pin mismatch fails.
- [x] All niucode and niuse gates run.
- [x] No registry write or version bump.
- [x] Script is executable and committed.
- [x] Preflight passes from a clean checkout.

## Notes

The script assumes sibling checkouts; `NIUCODE_DIR` overrides the path. CI can
run it once the Chrome smoke test is optional.
