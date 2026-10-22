#!/usr/bin/env python3
"""Read-only calendar MCP server for niuse.

Reads one iCalendar file (`CALENDAR_ICS`) and exposes:
  list_events(days)        events from CALENDAR_TODAY through today+days
  search_events(query)     case-insensitive text search

Times are reported as written, without timezone conversion. `CALENDAR_TODAY`
exists for deterministic tests; production uses the local date.
"""
import datetime
import json
import os
import sys
from pathlib import Path

try:
    ICS = Path(os.environ["CALENDAR_ICS"]).expanduser()
except KeyError:
    print("calendar: CALENDAR_ICS is required", file=sys.stderr)
    sys.exit(2)
if not ICS.is_file():
    print(f"calendar: CALENDAR_ICS is not a file: {ICS}", file=sys.stderr)
    sys.exit(2)

MAX_EVENTS = 100
TODAY = datetime.date.fromisoformat(
    os.environ.get("CALENDAR_TODAY", datetime.date.today().isoformat())
)


def send(payload):
    print(json.dumps(payload), flush=True)


def reply(request_id, result):
    send({"jsonrpc": "2.0", "id": request_id, "result": result})


def fail(request_id, message, code=-32602):
    send({"jsonrpc": "2.0", "id": request_id, "error": {"code": code, "message": message}})


def tool_output(request_id, text, ok=True):
    reply(request_id, {"content": [{"type": "text", "text": text}], "isError": not ok})


def unfold(text):
    lines = []
    for line in text.splitlines():
        if line.startswith((" ", "\t")) and lines:
            lines[-1] += line[1:]
        else:
            lines.append(line)
    return lines


def unescape(value):
    return (
        value.replace("\\n", "\n")
        .replace("\\N", "\n")
        .replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\\\", "\\")
    )


def parse_date(value):
    value = value.strip()
    if len(value) >= 8 and value[:8].isdigit():
        return datetime.date(int(value[0:4]), int(value[4:6]), int(value[6:8]))
    return None


def parse_time(value):
    value = value.strip().rstrip("Z")
    if "T" not in value:
        return None
    clock = value.split("T", 1)[1]
    if len(clock) >= 4 and clock[:4].isdigit():
        return f"{clock[0:2]}:{clock[2:4]}"
    return None


def load_events():
    events = []
    current = None
    for line in unfold(ICS.read_text(encoding="utf-8", errors="replace")):
        if line == "BEGIN:VEVENT":
            current = {}
            continue
        if line == "END:VEVENT":
            if current and current.get("date"):
                events.append(current)
            current = None
            continue
        if current is None or ":" not in line:
            continue
        key, value = line.split(":", 1)
        name = key.split(";", 1)[0].upper()
        if name == "DTSTART":
            current["date"] = parse_date(value)
            current["start"] = parse_time(value)
        elif name == "DTEND":
            current["end"] = parse_time(value)
            current["end_date"] = parse_date(value)
        elif name == "SUMMARY":
            current["summary"] = unescape(value).strip()
        elif name == "LOCATION":
            current["location"] = unescape(value).strip()
        elif name == "DESCRIPTION":
            current["description"] = unescape(value).strip()
        elif name == "UID":
            current["uid"] = value.strip()
    return events


def label(event):
    date = event["date"].isoformat()
    if event.get("start"):
        clock = event["start"]
        if event.get("end") and event.get("end_date") == event["date"]:
            clock += f"–{event['end']}"
        prefix = f"{date} {clock}"
    else:
        prefix = f"{date} (all day)"
    text = event.get("summary") or "(untitled)"
    if event.get("location"):
        text += f" @ {event['location']}"
    return f"{prefix} · {text}"


def list_events(arguments):
    days = min(max(int(arguments.get("days", 7)), 1), 365)
    end = TODAY + datetime.timedelta(days=days)
    events = [
        event
        for event in load_events()
        if TODAY <= event["date"] <= end
    ]
    events.sort(key=lambda event: (event["date"], event.get("start") or ""))
    if not events:
        return f"no events between {TODAY} and {end}"
    return "\n".join(label(event) for event in events[:MAX_EVENTS])


def search_events(arguments):
    query = str(arguments.get("query", "")).strip()
    if not query:
        raise ValueError("query is required")
    limit = min(max(int(arguments.get("limit", 10)), 1), MAX_EVENTS)
    needle = query.lower()
    hits = []
    for event in sorted(load_events(), key=lambda event: event["date"]):
        haystack = " ".join(
            [event.get("summary", ""), event.get("location", ""), event.get("description", "")]
        ).lower()
        if needle in haystack:
            hits.append(label(event))
            if len(hits) >= limit:
                break
    return "\n".join(hits) if hits else "no matches"


TOOLS = [
    {
        "name": "list_events",
        "description": "List calendar events from today through today+days. Read-only.",
        "inputSchema": {
            "type": "object",
            "properties": {"days": {"type": "integer", "minimum": 1, "maximum": 365}},
        },
    },
    {
        "name": "search_events",
        "description": "Search event summary, location, and description case-insensitively.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "limit": {"type": "integer", "minimum": 1, "maximum": MAX_EVENTS},
            },
            "required": ["query"],
        },
    },
]

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    message = json.loads(line)
    method = message.get("method")
    request_id = message.get("id")
    params = message.get("params") or {}
    if method == "initialize":
        reply(request_id, {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "niuse-calendar", "version": "1"},
        })
    elif method == "notifications/initialized":
        continue
    elif method == "tools/list":
        reply(request_id, {"tools": TOOLS})
    elif method == "tools/call":
        name = params.get("name")
        arguments = params.get("arguments") or {}
        try:
            if name == "list_events":
                tool_output(request_id, list_events(arguments))
            elif name == "search_events":
                tool_output(request_id, search_events(arguments))
            else:
                raise ValueError(f"unknown tool {name}")
        except (ValueError, OSError) as error:
            tool_output(request_id, str(error), ok=False)
    elif request_id is not None:
        fail(request_id, f"unknown method {method}", -32601)
