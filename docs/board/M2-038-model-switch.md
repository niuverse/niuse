+++
id = "M2-038"
title = "Switch models without restarting"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-002"]
docs = ["architecture"]
+++

## Scope

- `/model` prints the current model and provider.
- `/model <name>` rebuilds the provider for the same preset with the new model; the next turn uses it.
- `/model default` drops the override and returns to the provider's default model.
- A failed rebuild reports the error and keeps the previous model.
- The provider is only rebuilt between turns.

## Acceptance

- [x] `/model` and `/model default` print the expected model.
- [x] `default` is case-insensitive and covered by a unit test.
- [x] `/help` and completion list the command.
- [x] Switching does not rewrite the system prompt or the thread.

## Notes

Rebuilding the provider re-resolves the credential, which is the same path startup uses. No second provider configuration is stored.
