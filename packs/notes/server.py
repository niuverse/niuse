#!/usr/bin/env python3
"""Read-only notes MCP server for niuse.

Exposes three tools over stdio:
  search_notes(query, limit)  case-insensitive line search
  read_note(path)             read one note, bounded and confined to NOTES_DIR
  list_notes(limit)           recent notes by mtime

The server never writes. Paths are resolved before the confinement check, so a
symlink cannot escape NOTES_DIR.
"""
import json
import os
import sys
from pathlib import Path

try:
    ROOT = Path(os.environ["NOTES_DIR"]).expanduser().resolve()
except KeyError:
    print("notes: NOTES_DIR is required", file=sys.stderr)
    sys.exit(2)
if not ROOT.is_dir():
    print(f"notes: NOTES_DIR is not a directory: {ROOT}", file=sys.stderr)
    sys.exit(2)

MAX_READ = 20_000
MAX_HITS = 50
EXTENSIONS = (".md", ".txt")


def send(payload):
    print(json.dumps(payload), flush=True)


def reply(request_id, result):
    send({"jsonrpc": "2.0", "id": request_id, "result": result})


def fail(request_id, message, code=-32602):
    send({"jsonrpc": "2.0", "id": request_id, "error": {"code": code, "message": message}})


def tool_output(request_id, text, ok=True):
    reply(request_id, {"content": [{"type": "text", "text": text}], "isError": not ok})


def within(path):
    candidate = (ROOT / path).resolve()
    if candidate != ROOT and ROOT not in candidate.parents:
        raise ValueError("path escapes NOTES_DIR")
    return candidate


def notes():
    for path in ROOT.rglob("*"):
        if not path.is_file() or path.suffix.lower() not in EXTENSIONS:
            continue
        relative = path.relative_to(ROOT)
        if any(part.startswith(".") for part in relative.parts):
            continue
        yield path


def search_notes(arguments):
    query = str(arguments.get("query", "")).strip()
    if not query:
        raise ValueError("query is required")
    limit = min(int(arguments.get("limit", 10)), MAX_HITS)
    needle = query.lower()
    hits = []
    for path in sorted(notes()):
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for number, line in enumerate(text.splitlines(), 1):
            if needle in line.lower():
                hits.append(f"{path.relative_to(ROOT)}:{number}: {line.strip()[:240]}")
                if len(hits) >= limit:
                    return "\n".join(hits)
    return "\n".join(hits) if hits else "no matches"


def read_note(arguments):
    raw = str(arguments.get("path", "")).strip()
    if not raw:
        raise ValueError("path is required")
    path = within(raw)
    if not path.is_file():
        raise ValueError(f"not a note: {raw}")
    text = path.read_text(encoding="utf-8", errors="replace")
    if len(text) > MAX_READ:
        text = text[:MAX_READ] + "\n… truncated"
    return text


def list_notes(arguments):
    limit = min(int(arguments.get("limit", 20)), MAX_HITS)
    rows = []
    for path in notes():
        rows.append((path.stat().st_mtime, path))
    rows.sort(key=lambda item: item[0], reverse=True)
    lines = []
    for stamp, path in rows[:limit]:
        date = __import__("datetime").datetime.fromtimestamp(stamp).strftime("%Y-%m-%d")
        lines.append(f"{path.relative_to(ROOT)}  ({date})")
    return "\n".join(lines) if lines else "no notes"


TOOLS = [
    {
        "name": "search_notes",
        "description": "Search note text case-insensitively. Returns file:line hits within NOTES_DIR.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "limit": {"type": "integer", "minimum": 1, "maximum": MAX_HITS},
            },
            "required": ["query"],
        },
    },
    {
        "name": "read_note",
        "description": f"Read one UTF-8 note inside NOTES_DIR, at most {MAX_READ} characters.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    },
    {
        "name": "list_notes",
        "description": "List recent notes in NOTES_DIR by modification time.",
        "inputSchema": {
            "type": "object",
            "properties": {"limit": {"type": "integer", "minimum": 1, "maximum": MAX_HITS}},
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
            "serverInfo": {"name": "niuse-notes", "version": "1"},
        })
    elif method == "notifications/initialized":
        continue
    elif method == "tools/list":
        reply(request_id, {"tools": TOOLS})
    elif method == "tools/call":
        name = params.get("name")
        arguments = params.get("arguments") or {}
        try:
            if name == "search_notes":
                tool_output(request_id, search_notes(arguments))
            elif name == "read_note":
                tool_output(request_id, read_note(arguments))
            elif name == "list_notes":
                tool_output(request_id, list_notes(arguments))
            else:
                raise ValueError(f"unknown tool {name}")
        except (ValueError, OSError) as error:
            tool_output(request_id, str(error), ok=False)
    elif request_id is not None:
        fail(request_id, f"unknown method {method}", -32601)
