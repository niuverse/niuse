+++
id = "M9-008"
title = "Documentation link check"
status = "done"
milestone = "M9"
module = "dashboard"
size = "S"
priority = "P2"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M9-007"]
docs = ["roadmap", "evals"]
+++

## Problem

Docs and cards link to ADRs, plans, and each other. A rename can leave a dead
relative link that no test notices; two M4-002 links were already broken.

## Scope

- Scan `README.md`, `AGENTS.md`, and `docs/**/*.md`.
- Check relative links, skipping http, anchors, mailto, and paths outside the
  repo; ignore fenced code blocks.
- Fail on missing targets; run in `check.sh`.
- Deterministic self-test with a valid and a missing link.

## Acceptance

- [x] The two M4-002 links are fixed.
- [x] A valid relative link passes.
- [x] A missing relative link fails with its source.
- [x] Fenced code examples are ignored.
- [x] `check.sh` runs the live check and self-test.
- [x] External links and anchors are skipped.

## Notes

This checks existence, not anchors. A heading rename inside a valid file is not
detected; that would need a Markdown parser.
