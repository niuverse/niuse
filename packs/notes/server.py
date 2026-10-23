#!/usr/bin/env python3
"""Notes MCP server for niuse.

Read-only by default; exposes three tools over stdio:
  search_notes(query, limit)  case-insensitive line search
  read_note(path)             read one note, bounded and confined to NOTES_DIR
  list_notes(limit)           recent notes by mtime

With NOTES_WRITE=1 it also exposes:
  create_note(title, text)    create a new Markdown note, never overwrite

Paths are resolved before the confinement check, so a symlink cannot escape
NOTES_DIR. Write mode is opt-in through the child environment; a read-only
manifest must not expose the write tool.
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
MAX_WRITE = 20_000
MAX_HITS = 50
EXTENSIONS = (".md", ".txt")
WRITE = os.environ.get("NOTES_WRITE", "").strip().lower() in ("1", "true", "yes")


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


def create_note(arguments):
    if not WRITE:
        raise ValueError("write mode is disabled; set NOTES_WRITE=1")
    title = str(arguments.get("title", "")).strip()
    text = str(arguments.get("text", ""))
    if not title:
        raise ValueError("title is required")
    if not text:
        raise ValueError("text is required")
    if len(text) > MAX_WRITE:
        raise ValueError(f"text is longer than {MAX_WRITE} characters")
    slug = "".join(
        character if character.isalnum() or character in "-_" else "-"
        for character in title.lower()
    ).strip("-")
    slug = "-".join(part for part in slug.split("-") if part)[:60]
    if not slug:
        raise ValueError("title has no usable filename characters")
    path = within(f"{slug}.md")
    if path.exists():
        raise ValueError(f"note already exists: {path.name}")
    path.write_text(f"# {title}\n\n{text.rstrip()}\n", encoding="utf-8")
    return f"created {path.relative_to(ROOT)}"


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
if WRITE:
    TOOLS.append({
        "name": "create_note",
        "description": "Create a new Markdown note in NOTES_DIR. Refuses to overwrite.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "title": {"type": "string"},
                "text": {"type": "string"},
            },
            "required": ["title", "text"],
        },
    })

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
            elif name == "create_note":
                tool_output(request_id, create_note(arguments))
            else:
                raise ValueError(f"unknown tool {name}")
        except (ValueError, OSError) as error:
            tool_output(request_id, str(error), ok=False)
    elif request_id is not None:
        fail(request_id, f"unknown method {method}", -32601)
