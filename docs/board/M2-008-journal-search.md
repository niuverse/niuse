+++
id = "M2-008"
title = "Journal search"
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

- `niuse journal --search <query>` ranks journal bullets across all months.
- `/journal search <query>` does the same in the REPL.
- Ranking reuses the memory scorer: substring hits first, bigram similarity above 0.2.
- Results show the date and the bullet, capped at ten.

## Acceptance

- [x] A substring hit is found and keeps its date.
- [x] An unrelated query returns no rows.
- [x] The CLI and REPL paths were exercised.
- [x] Ranking and empty-result behaviour have unit tests.

## Notes

Memory answers "what is true"; the journal answers "when did we decide this". Search is what makes the second question cheap.
