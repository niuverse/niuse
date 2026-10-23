+++
id = "final-preflight"
title = "Final release preflight · 2026-10-23"
kind = "doc"
status = "done"
order = 26
summary = "Clean-tree cross-repository preflight passed at niucode 23e23af; publishing remains a human step and dogfood is still 2/10 days."
+++

## Result

`./scripts/release-preflight.sh` passed with exit 0 from clean trees.

- niucode `23e23af`, niuse `71c4ab8` (`niucode.rev` matched)
- niucode MSRV, tests, clippy, package, dashboard: pass
- niuse MSRV default/web/tui: pass
- `scripts/check.sh`: pass
- drift: `niucode matches pinned 23e23af`

## Dogfood gate

```text
dogfood days: 2/10
acceptance:   100% (1/1)
notes:        3 cited m-ids · 1 disagreement(s)
gap:          only 2/10 days
status:       in progress
```

No dogfood day was invented or backfilled. The calendar gate is still open.

## Release status

- Version bump: not applied.
- niucode 0.1.7: not published.
- niuse 0.1.13: not published.
- Publish dry runs: all eight niucode crates and both niuse default packages
  pass; `niuse --features web` still needs niucode 0.1.7.
- Human approval: still required.

## Verdict

The tree is release-ready technically, except for the intentionally gated
publication and the real dogfood calendar. Do not publish without explicit human
authorization.
