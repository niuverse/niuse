+++
id = "final-validation"
title = "Final validation · round 99"
kind = "doc"
status = "done"
order = 30
summary = "Clean-tree preflight passed at niucode beabb96 / niuse 440b03e; publication and dogfood remain the only open gates."
+++

## Result

`./scripts/release-preflight.sh` exited 0 from clean trees.

- niucode clean at `beabb96`
- niuse clean at `440b03e`
- `niucode.rev` matches `beabb9607b4120e21332cb68325ac7a8610e9702`
- niucode MSRV / tests / clippy / package / dashboard: pass
- niuse MSRV default/web/tui: pass
- `check.sh`: pass
- drift: matches
- board: 180 cards, 11 milestones, 3 active

## Dogfood

```text
dogfood days: 2/10
acceptance:   100% (1/1)
notes:        3 cited m-ids · 1 disagreement(s)
gap:          only 2/10 days
status:       in progress
```

No day was backfilled.

## Open gates

- Human approval to publish.
- Version bump for niucode 0.1.7 / niuse 0.1.13.
- Real dogfood calendar.

## Verdict

Technical objective and release readiness are verified. The remaining work is
human authorization and elapsed real dogfood days, not implementation.
