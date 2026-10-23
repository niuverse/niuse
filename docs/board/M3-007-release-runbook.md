+++
id = "M3-007"
title = "0.1.12 release runbook"
status = "done"
milestone = "M3"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M3-006"]
docs = ["release-012", "distribution"]
+++

## Problem

The 0.1.12 upload is a human-authorized action with a strict order: core first,
index wait, then the CLI. The exact commands and verification were spread
across the distribution doc and the publish script.

## Scope

- One runbook with preflight evidence, commands, verification, and failure
  handling.
- No upload in this card.
- Record the dry-run result and the dogfood-trend “insufficient data” behavior.

## Acceptance

- [x] The runbook names the authorization requirement.
- [x] Preflight evidence is dated and includes the core dry-run.
- [x] The core-first order and 20s index wait match the script.
- [x] Post-upload verification includes package and clean-root install.
- [x] Failure handling says bump, do not reuse.

## Notes

This card changes no code. It exists so the authorized upload is a single
reviewable step rather than an improvised sequence.
