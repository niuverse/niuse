+++
id = "M2-001"
title = "Ten working days of dogfood"
status = "ready"
milestone = "M2"
module = "repl"
size = "M"
priority = "P0"
updated = "2026-10-21"
depends_on = ["M0-003", "M0-005"]
docs = ["plan", "dogfood"]
+++

## Scope

- At least one real task per day goes through niuse, not niucode.
- Record proposals, acceptances, and concrete cases where memory saved a round of explanation.
- At least once it should disagree and change your mind.
- Keep the store under 50 entries; more means the bar is too low.

## Acceptance

- [ ] Ten consecutive working days.
- [ ] Candidate acceptance rate ≥ 60%.
- [ ] At least 3 correct citations of a past judgement.
- [ ] At least 1 case of being persuaded.
- [ ] At least 1 A/B case where `NIUSE_MEMORY=off` changes the answer.

## Notes

Record one row per day in [the dogfood log](../dogfood.md). This card writes no code. Its job is to stop new features before the current ones are validated.
