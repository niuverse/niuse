# Notes MCP pack

A small stdio MCP server over a notes directory. It exposes:

- `search_notes(query, limit)` — case-insensitive line search
- `read_note(path)` — one UTF-8 note, capped at 20k characters
- `list_notes(limit)` — recent notes by mtime

Read-only by default. With `NOTES_WRITE=1` it also exposes
`create_note(title, text)`, which writes one Markdown file and refuses to
overwrite. Every path is resolved before checking that it stays inside
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

## Write mode

Writes are opt-in and need both the child environment and the manifest:

```toml
[[server]]
name = "notes"
command = "python3"
args = ["/absolute/path/to/niuse/packs/notes/server.py"]
read_only = false
write_tools = ["create_note"]

[server.env]
NOTES_DIR = "/home/you/notes"
NOTES_WRITE = "1"
```

Run with `--allow-mcp-write` and an interactive approval mode. The first call
shows `mcp__notes__create_note` with a bounded argument preview; denial writes
nothing. `--approval never` and non-interactive runs do not mount this server.
