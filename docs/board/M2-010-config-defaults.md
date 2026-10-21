+++
id = "M2-010"
title = "Config file defaults"
status = "done"
milestone = "M2"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-003"]
docs = ["architecture"]
+++

## Scope

- `~/.niuse/config.toml` with a `[defaults]` table: provider, model, sandbox, approval, network, max_steps.
- Precedence is CLI flag → config file → built-in default.
- `niuse config init` writes a commented template; `config show` prints the file.
- Settings resolve lazily: offline commands (mem, loops, journal, export, status, stats) do not read the config, so a broken file cannot block them.
- Config files are `0600`.

## Acceptance

- [x] TOML parsing and the precedence rule have unit tests.
- [x] A config sandbox changes the REPL banner; a CLI flag overrides it.
- [x] A malformed config fails agent commands with the file path, but `status` still works.
- [x] Init refuses to overwrite without `--force`.

## Notes

Defaults belong in a file, not in muscle memory. Lazy resolution is deliberate: a typo in `config.toml` should not make `niuse mem show` unusable.
