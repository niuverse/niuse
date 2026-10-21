+++
id = "M2-074"
title = "Unknown config keys are rejected"
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

## Problem

`config.toml` was parsed without `deny_unknown_fields`, so `provder = "deepseek"` was silently ignored and the built-in default was used. The user sees the typo in the file and a different behavior at runtime.

## Fix

- `FileConfig` and `Defaults` deny unknown fields.
- `config effective`, `doctor`, and any provider-building command now fail with the TOML parse error instead of guessing.
- The settings test covers a misspelled field and an unknown top-level key.

## Acceptance

- [x] `provder` under `[defaults]` is a parse error.
- [x] An unknown top-level key is a parse error.
- [x] Empty and valid files still parse.
- [x] No runtime fallback hides the typo.

## Notes

A config file is a contract with the user; silently ignoring a line is the opposite of reviewable.
