+++
id = "M9-004"
title = "Combined personal workflow eval"
status = "done"
milestone = "M9"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M9-001"]
docs = ["evals", "roadmap"]
+++

## Problem

The eval suite tested each piece: recall, goals, plan injection, MCP packs,
verification. Nothing proved they compose into one personal workflow.

## Scope

- One agent turn with notes and calendar mounted together.
- A real plan step created through the store.
- The model calls both namespaced MCP tools.
- A step-tagged verification is appended, the step closes, and
  `niuse evidence --json` shows both.
- No API key, network, or real personal data.

## Acceptance

- [x] Both packs mount into one host.
- [x] Calendar search returns the standup.
- [x] Notes search returns `standup.md`.
- [x] The plan step closes only after the tagged verification.
- [x] Evidence JSON reports plan done, verification step, and ≥2 tools.
- [x] The eval runner includes the case.

## Notes

This is the cross-layer proof, not a model-quality claim. The provider is
scripted; the value is that the harness, two adapters, the plan store, and the
event log agree on one scenario.
