+++
id = "M2-013"
title = "Thread rotation"
status = "done"
milestone = "M2"
module = "repl"
size = "M"
priority = "P1"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-003"]
docs = ["architecture"]
+++

## Scope

- `/new` appends a session end, archives the current thread to `~/.niuse/threads/thread-<millis>.jsonl`, and opens a fresh default thread.
- `--thread PATH` resumes an archived thread without changing the default.
- The archive keeps the original JSONL untouched; history comes from the normal projection.
- `/status` reports the active thread, not the default path.

## Acceptance

- [x] `/new` archives the old file and creates a fresh three-event session.
- [x] `--thread` resumes an archive and reports its event count.
- [x] A missing archive source is a no-op.
- [x] Archive moves have unit tests.

## Notes

The default remains one continuous relationship. `/new` is a chapter break, not the normal path; old chapters stay readable and resumable.
