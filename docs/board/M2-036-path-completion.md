+++
id = "M2-036"
title = "Path completion for cd and export"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-033", "M2-035"]
docs = ["architecture"]
+++

## Scope

- Tab after `/cd ` lists directories only; Tab after `/export ` lists files and directories.
- Relative fragments resolve against the session's current working directory, including after `/cd`.
- `~/` fragments resolve against `$HOME`; a bare `~` completes to `~/`; absolute fragments against their own root.
- Directory replacements end with `/`; hidden entries appear only when the fragment starts with `.`.

## Acceptance

- [x] Directories and files are filtered and sorted.
- [x] `/cd` completion never offers a regular file.
- [x] The completer shares the live cwd with the REPL, so `/cd` takes effect immediately.
- [x] A bare `~` completes to `~/`.
- [x] Pure path helpers have unit tests.

## Notes

Path completion is the one place where the completer needs mutable state; an `Arc<Mutex<PathBuf>>` keeps it in sync without making `App` global.
