+++
id = "M2-022"
title = "Bootstrap profile draft"
status = "done"
milestone = "M2"
module = "bootstrap"
size = "M"
priority = "P1"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-005"]
docs = ["plan", "architecture"]
+++

## Scope

- `niuse bootstrap --profile` makes one extra model call and drafts `user.md` from the selected sessions.
- It only writes when `user.md` is missing or `--force-profile` is passed.
- Input is capped at 10k characters; output is asked for three sections and no speculation.
- `--dry-run` reports that a profile would be drafted without calling anything.
- The write is recorded in metrics.

## Acceptance

- [x] A fresh home gets a user.md draft; an existing one is skipped by default.
- [x] The input builder respects its cap (unit test).
- [x] The draft is written `0600` and the command suggests `niuse user edit`.
- [x] A real run produced a draft that admitted uncertainty instead of inventing facts.

## Notes

This is the missing half of the cold start described in the plan: candidates fill memory, the profile fills `user.md`. `identity.md` is still never mined.
