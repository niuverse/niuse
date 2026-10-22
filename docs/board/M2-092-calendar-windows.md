+++
id = "M2-092"
title = "Stats windows follow local calendar days"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-054"]
docs = ["architecture"]
+++

## Problem

After day labels moved to the local calendar, `stats --days 1` still used a rolling `now - 24h` cutoff. At 09:00 local it could show yesterday evening plus today, while claiming a one-day window; the labels and the window disagreed.

## Fix

- `Date::start_millis_local` returns local midnight for a calendar date, using the earliest valid instant across DST transitions.
- `window_start(days)` is `0` for all time, otherwise local midnight `days - 1` days ago.
- `stats` and `stats --trend` share that cutoff.
- Tests cover the local-midnight round trip and the one-day/two-day window starts; the CLI help says "local calendar days".

## Acceptance

- [x] `--days 1` starts at local midnight today.
- [x] `--days 2` starts at local midnight yesterday.
- [x] `--days 0` still means all time.
- [x] A DST transition cannot silently shift the cutoff by a day.

## Notes

A day label and a day window should agree; otherwise the trend looks like it has data from a day it filtered out.
