+++
id = "M6-003"
title = "Session digest into the journal"
status = "review"
milestone = "M6"
module = "cli"
size = "L"
priority = "P1"
updated = "2026-10-22"
depends_on = ["M6-001"]
docs = ["roadmap", "plan"]
+++

## Problem

The journal is written by the model from in-band blocks during a turn. When a
long thread ends, the decisions and turning points are still only in the
transcript; the next session starts without them. Re-reading the thread is not
a memory strategy.

## Scope

- Add an explicit `niuse digest [--thread PATH] [--dry-run]` that summarizes a
  finished thread into the journal, with source ids and no code dumps.
- Use a separate, tool-free model call with a strict prompt; never write on a
  failed parse.
- Keep the existing in-band journal as the default. The digest is opt-in until
  ten real digests are reviewed.
- Exclude tool output and secrets; run the same scan as every journal write.

## Acceptance

- [x] A scripted-provider test proves the summary is written once, uses only
      user/assistant text, and never writes tool output.
- [x] `--dry-run` prints the entry without writing.
- [x] A failed, empty, or malformed response writes nothing and says why.
- [ ] Ten real digests are reviewed before the path becomes automatic.
- [x] The journal format and existing day-merging behaviour are unchanged.

## Notes

The source thread is named in the command output and recorded in the `digest`
metric; the journal itself stays plain bullets. Do not summarize every turn: the
journal records change, not activity.
