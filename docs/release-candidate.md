+++
id = "release-candidate"
title = "Release candidate · 0.1.13"
kind = "doc"
status = "living"
order = 29
summary = "Release-candidate stop line: technical scope and preview/security work are green; publication and the dogfood calendar are human gates."
+++

## Status

Published 2026-10-23. This page is retained as the release-candidate record.

## Artifacts

- niucode: `0.1.7` (workspace currently `0.1.6`, pin `beabb96`)
- niuse: `0.1.13` (workspace currently `0.1.12`)
- No registry upload has occurred.

## Included experience

- Rich TUI Markdown: emphasis, code, links, lists/tasks, GFM alert labels and
  tones, tables, viewport rules, math fallback, syntax highlighting.
- Streaming/progress: caret, per-tool timers, aggregated timings, phase footer,
  reduced-motion envs.
- Local web: SSE chat, prompts, new/switch, context sidebar, search, export,
  copy, commands, themes, mobile overlay, keyboard shortcuts.
- Web Markdown: tables, tasks, footnotes, alerts, MathML, safe links/images.
- Web security: loopback binding, Host/Origin guards, CSP, token + HttpOnly
  cookie, body cap, read timeout, approvals fail closed, per-tool session allow.
- Boundary: default `niuse` does not link niucode; `tui`/`web` opt-in; pin and
  drift checks.

## Verification

- [Final preflight](evidence/final-preflight.md): clean-tree cross-repo pass.
- [Goal scope audit](evidence/goal-scope-audit.md): objective mapped to evidence.
- [Repo hygiene](evidence/repo-hygiene.md): clean and formatted.
- [Feature dry run](evidence/feature-dry-run.md): `web` needs 0.1.7 first.
- niucode publish dry run: all eight crates package/verify
  (in the niucode repository).

## Release order

1. Human approval.
2. Bump/publish niucode foundation crates, then `niucode` 0.1.7.
3. Verify `cargo install niucode --version 0.1.7`.
4. Bump niuse to 0.1.13, require `niucode = 0.1.7`.
5. Re-run `./scripts/release-preflight.sh` and the `web` feature dry run.
6. Publish niuse, update `niucode.rev`/compat.

## Gates

- Publication: **not authorized**, not performed.
- Dogfood: **2/10 real days**, no backfill.
- `niuse --features web` against registry 0.1.6 fails by design; 0.1.7 first.

## Stop line

This is a release candidate, not a release. Do not publish from this document
without explicit human approval.
