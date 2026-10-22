+++
id = "M6-007"
title = "Digest no-change pre-check"
status = "done"
milestone = "M6"
module = "cli"
size = "S"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M6-006"]
docs = ["roadmap", "evals"]
+++

## Problem

Tightening the digest prompt did not stop activity summaries: a routine
create-and-confirm task still produced two bullets. The model was using the
digest call to summarize what happened because that is what the call asks for.

## Scope

- One cheap YES/NO gate before the digest call.
- NO: exit 0, no journal, no metric, same output as explicit `NONE`.
- YES: run the existing digest.
- Anything else: error without writing.
- Tests for NO, YES, malformed gate, and the existing branches.

## Acceptance

- [x] A trivial session returns NO and writes nothing.
- [x] A routine session with an unresolved due loop returns YES; that is a real
      unresolved issue, not activity.
- [x] Gate NO records no digest metric.
- [x] Gate YES still caps at five bullets and scans secrets.
- [x] Malformed gate output fails closed.

## Notes

The gate costs one extra small model call per digest. It is opt-in and far
cheaper than writing an activity journal. The prompt-tightening change stays;
the gate is the reliable layer.
