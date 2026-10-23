+++
id = "evidence-experience-midpoint"
title = "Experience midpoint evidence"
kind = "evidence"
status = "living"
order = 90
summary = "Round-50 verification of the TUI rendering and local web experience work, with known limits and release blockers."
+++

## Snapshot

- Date: 2026-10-23
- niucode pin: `d8d078c`
- niuse: workspace 0.1.12 (not published from this tree)
- Goal rounds completed: 50 / 100

This is a checkpoint, not a claim that the whole 100-round goal is finished.

## Delivered

### TUI

- Inline Markdown: bold, italic, code, links with URL, strikethrough, math.
- Level-aware headings, ordered/unordered lists, tasks, quotes, rules.
- Buffered tables with header/border tone, cell inline styles, alignment and
  truncation.
- Inline and display math fallback.
- Code language labels and token highlighting, including multi-line block
  comments and triple-quoted strings.
- Streaming caret, per-tool elapsed footer, compact per-tool markers.
- Evidence/details paths and bounded volume tests remain intact.

### Web

- Loopback-only HTTP + chunked SSE; no remote assets or build step.
- Read-only/workspace-write turns, streamed Markdown, MathML, highlighted code.
- Browser approvals with structured diff previews; tool cards.
- Product context sidebar (niuse): identity/user/goals/plan/memory/loops/report,
  live refresh, filtering, collapsed-state persistence.
- Model/cwd/policy status, turn timer, token usage.
- Multi-line composer, prompt history, local commands and completion menu,
  `/clear`, `/new`, export, search, copy, safe auto-follow, theme and
  reduced-motion support.
- niuse `/new` archives the active thread before recreating `thread.jsonl`.

### Boundary

- niucode owns transport, protocol, and reference client.
- niuse owns product prompt/context and thread semantics.
- Default niuse binary does not link niucode; `tui` and `web` are separate
  opt-in features.

## Verification

```text
cargo +1.88.0 check --workspace --all-targets        niucode: clean
cargo package -p niucode --allow-dirty               niucode: verifies
cargo +1.88.0 check --workspace --all-targets        niuse: clean
cargo +1.88.0 check --features web                   niuse: clean
./scripts/check.sh                                   green
./scripts/check-niucode-drift.sh                     pinned d8d078c
cargo test --workspace                               green
cargo clippy --workspace --all-targets -- -D warnings green
```

## Known limits

- No remote mode, authentication, or TLS; loopback only by design.
- Web math uses native MathML, not KaTeX; complex TeX remains unsupported.
- Session switching is not exposed; `/new` reloads onto a fresh session and old
  SSE connections finish against the old file.
- Context refresh is a page request after `context_changed` plus a periodic
  fallback, not a continuous context stream.
- Release versions are not bumped or published from this tree; publication
  needs human approval and coordinated niucode/niuse ordering.
- TUI tool timing remains per-running-tool plus completion elapsed; there is no
  per-tool history table.

## Next release gates

1. Human approval.
2. Bump niucode workspace crates and publish foundation crates before the CLI.
3. Publish niucode, then niuse 0.1.13 requiring it.
4. Update `niucode.rev`, compat, dashboards, and release notes.
5. Keep the dogfood/acceptance gates as separate release inputs.
