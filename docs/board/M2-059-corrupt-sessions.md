+++
id = "M2-059"
title = "Corrupt session lines are tolerated"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-058"]
docs = ["architecture"]
+++

## Scope

- A session log with a bad JSON line loads its valid events; the loader warns on stderr.
- `sessions --json` still lists it, with the valid event count and a title from the first readable user message.
- `export --session` succeeds on the same file and writes a `0600` Markdown file.
- An end-to-end CLI test pins both behaviors.

## Acceptance

- [x] `events` counts only decodable lines.
- [x] The title comes from the readable user message.
- [x] Export exits zero and contains that message.
- [x] No new dependency.

## Notes

Session logs are append-only files that can be truncated by a crash or edited by hand. Listing and export should degrade to the readable prefix, not refuse the whole file.
