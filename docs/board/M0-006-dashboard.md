+++
id = "M0-006"
title = "Architecture dashboard"
status = "done"
milestone = "M0"
module = "dashboard"
size = "L"
priority = "P1"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-003"]
docs = ["architecture", "plan"]
+++

## Scope

- Reuse niucode's xtask generator and three-column shell, with a niuse project model.
- Four views: Architecture (default), Work, Docs, Decisions.
- Sources: `project.json` plus `board/*.md` plus `docs/**/*.md`.
- Single-file `docs/index.html`, offline, no CDN, no runtime fetch.
- `cargo xtask dashboard --check` fails when generated output is stale.

## Acceptance

- [x] `cargo xtask dashboard` writes `docs/index.html` and `model.json`.
- [x] Validation passes: module paths exist, card references resolve, ADR statuses are legal.
- [x] Architecture shows modules, layers, and dependency arrows.
- [x] Docs opens the architecture doc, the plan, and both ADRs.
- [x] `--check` passes.

## Notes

The board is read-only; state changes go through the repository so it never becomes a second source of truth. The generator is copied from niucode; only branding and the project model changed.
