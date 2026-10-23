+++
id = "repo-hygiene"
title = "Final repository hygiene · 2026-10-23"
kind = "doc"
status = "done"
order = 28
summary = "Both repos clean, formatted, free of debug leftovers and stray temp paths; pin matches the niucode checkout."
+++

## Checked

- `git status --short --untracked-files=all` in both repositories: empty.
- `cargo fmt --all --check`: pass in both.
- No `niucode-rule-debug` / `niucode-emit-debug` leftovers.
- No TODO/FIXME/XXX on the changed TUI/web surface or the reviewed cards.
- No `/tmp/` paths written into scripts or evidence.
- `niucode.rev` matched `23e23af`.

## Heads

- niucode `23e23af` — final visual audit
- niuse `be5bf35` — goal scope audit

## Not checked here

- Registry publication (human gate).
- Dogfood calendar (2/10, real days only).
- Installed binaries on the machine (they are not rebuilt by this audit).
