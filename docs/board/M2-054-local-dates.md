+++
id = "M2-054"
title = "Local dates near midnight"
status = "backlog"
milestone = "M2"
module = "stores"
size = "M"
priority = "P2"
updated = "2026-10-22"
depends_on = ["M0-001"]
docs = ["architecture"]
+++

## Problem

`Date::today()` is UTC. At 01:35 local (UTC+8) on 2026-10-22, a self-check wrote journal and memory entries dated 2026-10-21. For a daily companion, "today" disagreeing with the user's calendar for the first eight hours of the day is surprising.

## Options

- **Keep UTC and document it.** Zero code risk; dates remain comparable across machines. The mismatch stays visible near midnight.
- **Use the local offset.** Add `time` or `chrono` to `niuse-core`, derive the local date in `Date::today()`, and keep storage as `YYYY-MM-DD`. Test fixtures that assume UTC need a fixed offset or explicit dates.
- **Store the offset with each date.** Most faithful for travel, but changes the Markdown format and every reader.

## Not decided

This is a product call: correctness of the calendar vs. a dependency-light core and stable fixtures. Decision should land with the M2 dogfood notes, not as an incidental refactor.
