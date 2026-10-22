+++
id = "M6-009"
title = "Digest gate verdict and reason"
status = "done"
milestone = "M6"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M6-008"]
docs = ["roadmap", "evals"]
+++

## Problem

The YES/NO gate was auditable only by its verdict. A wrong YES or NO could not
be explained without another model call.

## Scope

- Gate returns one line: `YES: reason` or `NO: reason`.
- Parse verdict case-insensitively; a bare `YES`/`NO` is still accepted with a
  placeholder reason; anything else fails closed.
- Print the reason on YES and on NO (dry-run and real).
- Increase the gate token cap to fit one sentence.
- Parse tests for YES, NO, bare verdict, and malformed output.

## Acceptance

- [x] YES verdicts print their reason before the digest.
- [x] NO verdicts include the reason in the no-change message.
- [x] Bare legacy verdicts still work.
- [x] Malformed verdicts write nothing.
- [x] Real validation: RSS → YES with a reason; trivial → NO with a reason.

## Notes

Reasons are model prose and can be wrong; the verdict and its reason are shown
together so the human can judge. Reasons are not stored in the journal or
metrics.
