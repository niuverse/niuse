+++
id = "M2-088"
title = "Effective config as JSON"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-028"]
docs = ["architecture"]
+++

## Scope

- `niuse config effective --json` emits provider, model, sandbox, approval, network, and max_steps.
- A missing model is `null`, matching the runtime's provider-default meaning.
- The text view is unchanged when the flag is absent.
- An end-to-end CLI test parses the object and checks defaults.

## Acceptance

- [x] JSON parses and carries the six settings.
- [x] `model` is null when unset.
- [x] `max_steps` stays numeric.
- [x] No provider or credential is constructed.

## Notes

`config effective` now matches the machine-readable style of the rest of the CLI.
