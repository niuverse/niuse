+++
id = "M6-003"
title = "Session digest into the journal"
status = "backlog"
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

- [ ] A scripted-provider test proves the summary is written once, with source
      ids and no raw tool output.
- [ ] `--dry-run` prints the entry without writing.
- [ ] A failed or malformed response writes nothing and says why.
- [ ] Ten real digests are reviewed before the path becomes automatic.
- [ ] The journal format and existing day-merging behaviour are unchanged.

## Notes

Do not summarize every turn. The journal records change, not activity.
