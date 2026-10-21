+++
id = "M2-028"
title = "Effective config view"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M2-010"]
docs = ["architecture"]
+++

## Scope

- `niuse config effective` prints the merged values: provider, model, sandbox, approval, network, max_steps.
- Precedence is the same as runtime: CLI flag, then config file, then built-in.
- `max_steps = 0` is shown as `unlimited`; a missing model is shown as the provider default.
- No provider or credential is constructed.
- The doctor hint now points at `bootstrap --profile` and `user edit`.

## Acceptance

- [x] Defaults print on a clean home.
- [x] CLI flags override the file in the printed view.
- [x] An end-to-end CLI test asserts the override.
- [x] The command works without network or credentials.

## Notes

The raw file answers "what did I configure"; this answers "what will actually run".
