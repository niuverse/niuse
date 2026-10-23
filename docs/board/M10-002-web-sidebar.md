+++
id = "M10-002"
title = "Product context sidebar"
status = "done"
milestone = "M10"
module = "cli"
size = "M"
priority = "P0"
updated = "2026-10-23"
completed = "2026-10-23"
depends_on = ["M10-001"]
docs = ["roadmap"]
+++

## Problem

`niuse web` had the product prompt but the page still looked like generic
niucode. Identity, user, goals, plan, memory, and loops were invisible.

## Scope

- Build a bounded context JSON from the seven stores.
- Pass brand/title and generic sections to `niucode::web::Options`.
- Use the new `/api/context` endpoint and responsive sidebar.
- Keep the default binary free of niucode; gate web behind `--features web`.
- Do not fork niucode's HTML.

## Acceptance

- [x] `niuse web --features web` compiles.
- [x] Sections cover identity, user, goals, plan, memory, and open loops.
- [x] Brand/title become niuse.
- [x] Content is escaped by `textContent` in the client.
- [x] Default `cargo check` does not link niucode.
- [x] niucode's `/api/context` route has a loopback test.

## Notes

The context is a snapshot per page load; it is not pushed over SSE yet. Report/
dogfood panels and edits are later slices.
