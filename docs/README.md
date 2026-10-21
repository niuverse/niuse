+++
id = "docs-readme"
title = "Dashboard guide"
kind = "guide"
status = "living"
order = 90
summary = "Sources, generation commands, the four views, and the ritual for adding modules, cards, and docs."
+++

## What this is

`docs/index.html` is the project's only architecture reading surface: Architecture, Work, Docs, and Decisions share one three-column shell. It is generated from repository sources, opens offline, and needs no server.

```sh
cargo xtask dashboard          # writes docs/index.html and docs/dashboard/model.json
cargo xtask dashboard --check  # CI: fails when generated output is stale or references break
```

Generation is deterministic: the same inputs produce the same HTML. `receipt.json` records the build time, git revision, and input list, but is not part of the staleness check.

## Four views

- **Architecture (default)**: modules by layer, dependency arrows, contracts, and source. Selecting a module focuses its relations; status only shows implementation state, not progress.
- **Work**: one file per card, a handoff surface filtered by milestone. No completion rates, WIP counts, or progress bars.
- **Docs**: a Markdown index with a reading pane; supports deep links such as `#/docs/adr-0001`.
- **Decisions**: ADR cards plus a `kind = "decision"` timeline. Completion logs do not belong here.

## Sources

```text
docs/
  index.html                # generated, committed, openable via file://
  dashboard/
    project.json            # project, milestones, layers, modules, relations, decisions
    template.html           # single-file shell
    style.css               # three-column layout and views
    app.js                  # framework-free rendering
    model.json              # generated
    receipt.json            # build receipt, gitignored
  board/
    M0-001-five-stores.md   # one card per file
  adr/
    0001-memory-layer.md
    0002-identity-layer.md
    0003-kernel-product-boundary.md
  architecture.md
  compat.md
  plan.md
```

## Adding things

**Add a module**: add an entry to `modules` in `project.json`. Its `layer` must exist and `paths` must point at real files; any module not marked `planned` needs at least one existing path.

**Add a work card**: create a Markdown file under `docs/board/`. The TOML frontmatter must include `id`, `title`, `status`, `milestone`, `size`, and `updated`; `done` requires `completed`, and `blocked` requires `blocked_reason`.

**Add a doc**: any `.md` under `docs/` with TOML frontmatter between `+++` lines, including at least `id`, `title`, and `kind`. ADR `status` accepts only `proposed`, `accepted`, `superseded`, `rejected`, or `draft`.

**Add a decision**: write an ADR, or add a `kind = "decision"` event to `project.json`. When doing both, link them with `refs`.

## Discipline

The board is read-only; the browser cannot write back. State changes go through the repository, so the board never becomes a second source of truth. Run `--check` before committing.
