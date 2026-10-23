+++
id = "preflight-014"
title = "0.1.14 pre-publication preflight"
kind = "evidence"
status = "living"
order = 92
summary = "Source, MSRV, tests, clippy, dashboard, product gates, and drift pass before niucode 0.1.8 is published; package verification is the only skipped step."
+++

## Command

```sh
./scripts/release-preflight.sh --skip-package
```

Run from a clean `niuse` tree pinned to niucode `b92ca47`.

## Result · 2026-10-24

```text
== repositories ==
niucode clean at b92ca47
niuse clean at a46b226
== pin ==
pin matches b92ca47
...
== drift ==
niucode matches pinned 6fae7f5
preflight passed; publishing is still a separate human step
```

Covered:

- niucode MSRV, workspace tests, clippy, dashboard
- niuse MSRV for default/tui/web (also checked after the audit-event slices)
- `./scripts/check.sh`
- `niucode.rev` drift
- `cargo package` for `niuse-core` (part of check)
- board, links, dashboard, pack assets

Skipped:

- `cargo package -p niucode` because the registry does not yet contain
  `niucode-agent 0.1.8`; the full preflight is expected to pass only after the
  kernel publish order completes.

Re-run 2026-10-24 after the computer audit/frame-cache slices: passed with the
same result.

## Not done

No `cargo publish` ran. Human approval remains the release gate.
