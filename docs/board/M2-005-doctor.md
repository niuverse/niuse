+++
id = "M2-005"
title = "Doctor health check"
status = "done"
milestone = "M2"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-001"]
docs = ["architecture"]
+++

## Scope

- `niuse doctor` and `/doctor` check the home directory and mode, identity, user profile, memory counts, thread readability, metrics, credentials, bwrap, and the niucode pin.
- Offline: no model call, no network.
- Failures return a non-zero exit; warnings do not.
- The pin check only runs inside a checkout where `niucode.rev` exists.

## Acceptance

- [x] A healthy store reports zero failures.
- [x] Missing credentials are a failure; missing identity or bwrap is a warning.
- [x] A drifted sibling checkout is reported with the sync command.
- [x] Report counters have a unit test.

## Notes

Doctor is the onboarding answer to "why will it not run": one command, explicit levels, and the exact next step.
