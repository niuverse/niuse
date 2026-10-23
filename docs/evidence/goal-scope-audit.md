+++
id = "goal-scope-audit"
title = "Goal scope audit · TUI, streaming, web, boundary"
kind = "doc"
status = "done"
order = 27
summary = "Line-by-line audit of the 100-round objective against shipped code, tests, ADRs, and board cards; technical scope is complete, release/dogfood gates remain human."
+++

## Objective, line by line

> rich Markdown rendering in the TUI (bold, italic, inline code, links, lists,
> tables, math fallback)

- Inline emphasis/code/link styling: `markdown.rs`, ADR-0035, M9-001/002/003.
- Lists, tasks, GFM alert labels/tones: M9-018, M9-080, M9-081.
- Tables with alignment, wide-cell wrapping, inline styles: `table.rs`, M9-016/017.
- Math Unicode fallback: M9-020; PTY validates `α + β = γ`.
- Heading hierarchy, quote bars, viewport rules: M9-019, M9-059.
- PTY regression covers headings, emphasis, tables, code, math, tasks, alerts:
  M9-084.

> polished streaming/animation and progress states

- Streaming caret, running-tool timer, phase footer, unique tool markers,
  aggregated timings: `live.rs`, M9-025/026/027.
- Reduced motion env: M9-058; `NO_COLOR`/`TERM=dumb` fallbacks: M9-005.
- Web streaming throttle and final render: M9-078.

> a local web interaction surface comparable to a modern agent harness

- Loopback server, SSE, prompt/new/switch, approvals with diffs, tool cards,
  export, context sidebar, search, commands, themes, mobile, keyboard access:
  M9-007 through M9-074.
- Security: raw HTML escaping, href allowlist, image blocking, Host/Origin guards,
  CSP, per-run token + HttpOnly cookie, body cap, read timeout, threat model,
  publish dry runs.

> keeping the kernel/product boundary coherent

- niucode owns the harness/web/TUI; niuse composes identity/memory/prompt.
- Default niuse binary does not link niucode; `tui` and `web` are opt-in.
- `niucode.rev` pin + drift + sync script; compat matrix; no source fork.

> shipping verified, committed, pushed increments with tests, ADRs, roadmaps,
> and dashboards for up to 100 rounds

- Every round has a pushed commit in both repos where code changed.
- `check.sh`, MSRV, clippy, package, dashboard, drift, PTY, HTTP tests all run.
- ADR-0035/0036, rendering.md, roadmap/status, board cards, dashboard are kept
  current.

## Remaining, explicitly not claimed

- **Publishing**: versions are still 0.1.6/0.1.12; no registry upload; human
  authorization required.
- **Dogfood calendar**: 2/10 real days; no backfill.
- **Release bump**: not applied; `niuse --features web` dry run correctly fails
  against registry 0.1.6 until 0.1.7 exists.

## Verdict

The technical experience objective is met and evidenced. The two open gates are
human/calendar gates, not unused implementation work.
