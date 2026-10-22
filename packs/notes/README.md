# Read-only notes MCP pack

A small stdio MCP server over a notes directory. It exposes:

- `search_notes(query, limit)` — case-insensitive line search
- `read_note(path)` — one UTF-8 note, capped at 20k characters
- `list_notes(limit)` — recent notes by mtime

It never writes and resolves every path before checking that it stays inside
`NOTES_DIR`.

## Configure

Add this to `~/.niucode/mcp.toml`:

```toml
[[server]]
name = "notes"
command = "python3"
args = ["/absolute/path/to/niuse/packs/notes/server.py"]
read_only = true

[server.env]
NOTES_DIR = "/home/you/notes"
```

Then `niuse tui`, `niuse`, or `niuse exec` mounts it. Ask the agent to search
your notes; the tools appear as `mcp__notes__search_notes`, and so on.
