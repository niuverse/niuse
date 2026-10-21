+++
id = "M2-011"
title = "Manual journal entries and identity editing"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P2"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-001"]
docs = ["architecture"]
+++

## Scope

- `niuse journal --add <text>` and `/journal add <text>` append a bullet to today.
- `niuse identity show|edit` and `/identity edit` open the identity file in `$EDITOR`.
- Journal entries are secret-scanned and recorded in metrics.
- Editing uses a multiline helper; memory and candidate editing keep the one-line normaliser.

## Acceptance

- [x] A manual journal entry appears in `recent_journal` and in search.
- [x] Identity editing writes the file and keeps it `0600`.
- [x] Empty edits report "no change" and do not write.
- [x] The journal path has a unit test; identity editing was exercised with a scripted editor.

## Notes

Not every durable line comes from the model. Manual entry is the escape hatch when a block was not emitted.
