+++
id = "M2-001"
title = "Ten working days of dogfood"
status = "doing"
milestone = "M2"
module = "repl"
size = "M"
priority = "P0"
updated = "2026-10-22"
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
- [x] At least 1 case of being persuaded.
- [x] At least 1 A/B case where `NIUSE_MEMORY=off` changes the answer.

## Notes

2026-10-22: started. A real read-only `niuse exec` on this repository cited
README.md:106 for `niuse report`; the turn used 2 steps, 11k input, 76 output,
3.5k cached tokens. A second real turn used the notes MCP pack:
`mcp__notes__search_notes` returned `launch.md:1: Launch checklist` in 2 steps,
5k input, 95 output, 3.1k cached. A third real turn ran through `niuse tui` and
replied `391` (13k input, 2 output, 0 cached), confirming the TUI path works
against the real provider. A real write task used `--approval ask`: the diff
was shown, the write was approved, `hello.txt` contained `real write ok`, and
the answer surfaced a due loop from `loops.md` (3 steps, 7.1k input, 279 output,
6.3k cached). A real memory A/B used the same prompt twice: with memory on the
answer cited `m-3d0b`; with `NIUSE_MEMORY=off` it replied `NO_INJECTED_MEMORY`
(1 step each, 3.5k / 3.4k input). On 2026-10-22 I proposed a third RSS pack; the
companion argued against it (network egress, M8 already met with two packs, the
real blockers were 1/10 dogfood, zero digest reviews, and the unsigned
ADR-0034). I dropped the pack and accepted the dogfood/digest/ADR order. The
dogfood table now has 2/10 days and 1 cited id; eight more days and two more
citations remain open.

Record one row per day in [the dogfood log](../dogfood.md). This card writes no code. Its job is to stop new features before the current ones are validated.
