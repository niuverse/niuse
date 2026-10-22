+++
id = "M2-054"
title = "Local dates for the human calendar"
status = "done"
milestone = "M2"
module = "stores"
size = "M"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M0-001"]
docs = ["architecture", "plan"]
+++

## Problem

`Date::today()` was UTC. At 01:35 local (UTC+8) on 2026-10-22, a self-check wrote journal and memory entries dated 2026-10-21. For a daily companion, "today" disagreeing with the user's calendar for the first eight hours of the day is surprising.

## Decision

- **User-facing dates follow the machine's local calendar.** `Date::today()` uses `chrono::Local`; journal and memory labels are local dates.
- **Metrics trends group by local day too.** `Date::from_millis_local` maps epoch milliseconds to the local calendar day, so a late-evening turn appears on the day the user experienced it.
- **Event timestamps stay UTC epoch milliseconds.** Session logs and `metrics.jsonl` remain machine-comparable; only the derived day label is local.
- Storage stays `YYYY-MM-DD`; no offset is written into the Markdown.

## Why local over UTC

The date is a human-facing label: "what did I do today", "when did I last confirm this". UTC is the right choice for ordering events and comparing machines, not for the calendar the user reads. Frameworks that store UTC timestamps usually still render dates in the user's timezone; this is that split, without adding a timezone field per entry.

## Consequences

- Two machines in different timezones can label the same instant differently; that is the intended behavior for daily notes.
- Tests use explicit dates except `Date::today` and the millis-mapping test, which compare against `chrono::Local`.
- `niuse-core` gains `chrono` with the `clock` feature; MSRV stays 1.88.
- Metrics day keys are local; the underlying timestamps are unchanged.

## Acceptance

- [x] A memory added at 09:24 local on 2026-10-22 is dated 2026-10-22.
- [x] `Date::today` matches `chrono::Local`'s date.
- [x] `Date::from_millis_local` matches the local calendar day.
- [x] `stats --trend` groups metrics by local day.
- [x] README no longer claims UTC dates.
