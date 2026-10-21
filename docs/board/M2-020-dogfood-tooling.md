+++
id = "M2-020"
title = "Dogfood record tooling"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M2-004"]
docs = ["dogfood", "plan"]
+++

## Scope

- `niuse stats --json` emits a stable summary plus per-day rows for scripting.
- `docs/dogfood.md` defines the ten-day ritual: one row per day, two numbers, one sentence.
- The JSON includes turns, candidate decisions, acceptance, activity counts, and days.
- `/stats` keeps the human summary; `--json` is CLI-only.

## Acceptance

- [x] JSON output parses and includes `acceptance_percent` and `days`.
- [x] Serialization has a unit test with synthetic metrics.
- [x] The dogfood guide links the exact command and a table template.
- [x] README points at the guide.

## Notes

The point of the JSON is not dashboards; it is making the daily row cheap enough to actually write.
