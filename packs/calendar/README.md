# Read-only calendar MCP pack

A small stdio MCP server over one iCalendar file. It exposes:

- `list_events(days)` — from `CALENDAR_TODAY` through today + days
- `search_events(query, limit)` — summary, location, and description

It never writes and needs no network. Times are reported as written; there is
no timezone conversion. `CALENDAR_TODAY` exists for deterministic tests and
defaults to the local date.

## Configure

```toml
[[server]]
name = "calendar"
command = "python3"
args = ["/absolute/path/to/niuse/packs/calendar/server.py"]
read_only = true

[server.env]
CALENDAR_ICS = "/home/you/calendar.ics"
```

Then ask the agent what is coming up. Tools appear as
`mcp__calendar__list_events` and `mcp__calendar__search_events`.
