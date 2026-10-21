+++
id = "M0-004"
title = "Soft expiry, conflicts, secret scan"
status = "done"
milestone = "M0"
module = "policy"
size = "M"
priority = "P0"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-001"]
docs = ["adr-0001"]
+++

## Scope

- Normal injection within 30 days; unverified from 31 to 90; not injected after 90.
- Conflict detection via character-bigram Jaccard: ≥ 0.5 is similar; same tag lowers the bar to 0.35.
- Secret patterns: `sk-`, `ghp_`, `github_pat_`, `AKIA`, private-key blocks, Slack, Google, long hex.
- Refusal is the default; the CLI needs `--force` to override.

## Acceptance

- [x] Common key shapes match; ordinary Chinese prose and short hex do not.
- [x] Tests for similar, unrelated, and same-tag pairs.
- [x] A real `sk-...` input is refused with exit code 1.
- [x] Expired memory is excluded from the prompt.

## Notes

Scanning is a backstop, not a guarantee. Private projects, client names, and internal addresses still rely on human judgement at confirmation time.
