+++
id = "M2-014"
title = "Piped output exits cleanly"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-003"]
docs = ["architecture"]
+++

## Scope

- `niuse ... | head` no longer panics when the reader closes early; the process exits 0.
- Only the "Broken pipe" panic is intercepted; every other panic goes to the default hook unchanged.
- The fix is a panic hook rather than a global SIGPIPE reset, because SIGPIPE also covers socket writes made by the provider.

## Acceptance

- [x] `niuse mem list | head -1` exits 0 with no panic output.
- [x] `niuse status | head -1` behaves the same.
- [x] Non-pipe panics still print through the default hook.
- [x] `niuse status --thread <archive>` reports the requested thread.

## Notes

Small, but it was the first papercut hit while testing; a CLI that panics when paged is not pleasant to script against.
