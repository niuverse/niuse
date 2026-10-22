+++
id = "M9-001"
title = "Offline personal-agent eval"
status = "doing"
milestone = "M9"
module = "cli"
size = "M"
priority = "P1"
updated = "2026-10-22"
docs = ["roadmap", "evals"]
+++

## Problem

The personal agent changes through prompts, stores, tools, and MCP packs, but
there was no named offline suite that says which capability broke. Unit tests
cover modules; they do not cover the prompt → turn → answer chain as one case.

## Scope

- Named cases for memory injection, goal lifecycle, and the startup brief.
- A real MCP pack case in the same runner.
- A behavioural memory-on/off A/B, not just a string comparison.
- `./scripts/eval.sh` as the single command.
- Document what the suite does and does not prove.

## Acceptance

- [x] Memory-on and memory-off produce different answers, and only the on case
      cites the remembered text.
- [x] Open goals are injected; closed goals are not.
- [x] The brief reports overdue promises, stale memory, and the inbox.
- [x] The notes MCP pack is part of the runner.
- [x] `./scripts/eval.sh` passes offline.
- [ ] Recall through `memory_search`, MCP timeout/refusal, and a dogfood trend
      gate are added.

## Notes

Cases use a scripted provider, so this is a plumbing and prompt regression
suite. Real usefulness still needs dogfood.
