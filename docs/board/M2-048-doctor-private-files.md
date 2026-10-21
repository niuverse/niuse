+++
id = "M2-048"
title = "Doctor checks private files"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-005", "M2-031"]
docs = ["architecture"]
+++

## Scope

- `doctor` checks `~/.niuse/history` (expected `0600`) and `~/.niuse/backups/` (expected `0700`).
- Missing paths report `not created yet` rather than a failure.
- Group- or world-readable modes produce a warning with the exact `chmod` hint.
- The mode predicate has a unit test; no new permission repair is attempted automatically.

## Acceptance

- [x] A `0600` history reports ok.
- [x] A `0644` history produces a warning naming the expected mode.
- [x] Absent history/backups are not warnings.
- [x] `is_private_mode` is tested across private and leaky modes.

## Notes

History contains prompts, which can contain anything. Being unreadable by other local users is part of the privacy story, so it belongs in the same check as the stores.
