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

2026-10-22: reviewed three real `--dry-run` outputs:

- write session (`1792683203856-421044-0`): 4 bullets, task, decision, and due
  loop survived; no code dump.
- MCP session (`1792683062106-403780-0`): 2 bullets, source id, no raw tool
  output. Quality caveat: it summarizes activity rather than a change; a short
  successful task has little to distill.
- RSS disagreement session (`1792683287429-436740-0`): 5 bullets, captures the
  decision, three reasons, and the newly identified network-egress risk.

- routine session (`1792683461440-469411-0`): 2 bullets, still activity-only
  after prompt tightening. The session also contained an unresolved due loop,
  so the gate correctly said YES; activity-only output remains a known limit.
- trivial session (`1792683490175-471215-0`): the new pre-check said NO, so no
  digest call and no journal write.
- old-loop routine (`1792683461440-469411-0`), after the gate revision: NO,
  because the only "unresolved" item was a pre-existing loop reminder.
- RSS decision session, re-run: YES, 3 bullets (down from 5, more concise).

The gate now prints a one-line reason with every verdict. Real validation:
RSS → `YES · 助手明确反对...`; trivial → `NO · 用户仅要求回复数字391...`.

That is 4/10 real reviews; the path stays opt-in. Activity-only digests are a
known limitation, not a reason to auto-write.

The source thread is named in the command output and recorded in the `digest`
metric; the journal itself stays plain bullets. Do not summarize every turn: the
journal records change, not activity.
