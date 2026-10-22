+++
id = "M6-002"
title = "Project registry"
status = "done"
milestone = "M6"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M6-001"]
docs = ["roadmap", "architecture", "compat"]
+++

## Problem

The companion can remember a preference but cannot answer "what was I doing last
week, and in which repository?" niucode already stores per-project sessions
under `~/.niucode/projects/*/project.toml`; niuse neither reads nor surfaces
them.

## Scope

- Read the niucode project registry read-only: recorded working directory, last
  session activity, and session count.
- Add `niuse projects [--limit N] [--json]`, sorted by most recent activity.
- Show the current project and its most recent thread in `niuse status`.
- Tolerate missing, corrupt, or moved project directories; never write there.
- Keep the event log as the authority for a session; this is a directory view,
  not a second index.

## Acceptance

- [x] A project appears once, with its recorded path and latest activity.
- [x] Unreadable entries are skipped with a warning; a missing cwd is flagged
      and hidden from the default view.
- [x] `--json` is stable and has no absolute paths beyond the recorded cwd.
- [x] Tests cover ordering, filtering, limits, corrupt entries, and an empty
      registry.
- [x] No files under `~/.niucode` are created or modified.

## Notes

This is the data source for a future "what changed while I was away" brief; do
not add that behaviour here.
