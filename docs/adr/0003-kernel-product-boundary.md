+++
id = "adr-0003"
title = "ADR-0003: niucode is the kernel and UI; niuse is the product"
kind = "adr"
status = "accepted"
order = 22
summary = "Keep two repositories. Upstream a reusable CLI library and turn hooks; niuse owns identity, memory, policy, and product defaults. Merge only if the product layer becomes thin enough that separate releases stop mattering."
+++

## Status

Accepted · 2026-10-21.

Upstream proposal: [niucode ADR-0032](https://github.com/niuverse/niucode/blob/main/docs/adr/0032-embedding-library-surface.md) · [niuverse/niucode#1](https://github.com/niuverse/niucode/issues/1).

## Context

At the time of writing, niuse drove niucode's turn loop itself and owned a small renderer, because `niucode-cli` was a binary with no library target. ADR-0032 has since added that library target and exported `TurnHooks`; the hook invocation and renderer migration are still ahead. Reusing niucode's TUI would remove the duplicate front end, which raises the obvious question: if niuse reuses the TUI, why not merge it into niucode?

Two facts make the merge tempting:

- The shared surface is large: loop, session, tools, renderer, approval UI, diff preview.
- Every shared change currently costs a revision bump and a contract test.

Two facts push the other way:

- niucode's positioning is a **minimal, observable coding harness** that deliberately refuses long-term memory, resident services, and product defaults. Merging a companion into that repo changes what niucode is.
- The two projects have different release cadences and different failure modes: kernel breakage versus product annoyance.

## Decision

1. **Keep two repositories.** niucode is the kernel and the reference UI; niuse is the product layer and the identity.
2. **Move the front end upstream, not the product.** `niucode-cli` should expose a reusable library (TUI, runtime entry) plus a small turn-hook surface:
   - augment the system prompt before a turn;
   - observe a finished turn (for candidate extraction);
   - receive session lifecycle events.
   niuse implements the hooks and deletes its own renderer once the shared TUI is usable.
3. **Keep the product in niuse**: identity, user model, memory, journal, loops, protocol blocks, expiry/conflict/privacy policy, product defaults (model, permissions, persona).
4. **Pin what you consume.** `niucode.rev` fixes the revision required CI and deployments build against. The pin moves at most weekly through `scripts/sync-niucode.sh`; a nightly job tests niucode `main` so drift is visible early.
5. **Upstream first.** Any capability that a generic niucode user would want lands in niucode first. niuse never forks kernel files.
6. **Set a merge trigger instead of debating forever.** After the library split lands, if niuse's product-only surface falls below roughly 1,000 lines and separate releases stop mattering, merging becomes reasonable. Until then, two repos buy a clean kernel story and independent cadence.

## Consequences

**Better:**

- Reusing the TUI removes the largest duplication without diluting niucode's positioning.
- The contract test plus pinned revision turns "did upstream break us?" into a CI signal.
- Product decisions and kernel decisions keep separate ADR trails and release notes.

**Worse:**

- niucode must grow a library API and hooks, which is real design work and must stay generic.
- niuse still carries a binary, product defaults, and the hook implementation.
- The weekly bump is a recurring chore, even if a small one.

**Accepted trade-off:** a little ceremony now beats either a fork or a premature monorepo. If the ceremony stops paying for itself, the merge trigger is explicit.

## Alternatives

| Option | Rejected because |
|---|---|
| Merge niuse into niucode now | Changes what niucode is, couples releases, and mixes two boards and two audiences before the boundary is settled |
| Add `--profile personal` to niucode | Hides a product inside the kernel; the personal layer would inherit kernel release pressure and vice versa |
| Fork niucode inside niuse | Duplicates the loop and the front end, and guarantees drift |
| Keep the duplicate renderer forever | Cheapest this month, most expensive this year; every TUI improvement has to be rebuilt twice |
