+++
id = "release-013"
title = "Release 0.1.13 (draft)"
kind = "doc"
status = "draft"
order = 32
summary = "Draft notes for niuse 0.1.13: opt-in web companion, shared TUI rendering, and verification evidence."
+++

## Status

Draft. Not published. The workspace is still 0.1.12. Publishing requires human
approval and a coordinated `niucode 0.1.7` release first.

## Why 0.1.13

`niuse web` needs `niucode::web` and `Options.system_prompt`, which are newer
than niucode 0.1.6 on crates.io. The web feature is opt-in, preserving the rule
that the default binary does not link niucode's TUI.

## Highlights

- **`niuse web --features web`**: loopback-only browser companion using the
  niuse product prompt (identity, user, goals, plan, memory, loops, journal).
  Startup prints a random token URL; page/API/SSE are gated behind an HttpOnly
  cookie.
- **Live context**: the sidebar re-reads the Markdown stores per request and
  refreshes after turns/reports.
- **Product report**: the 7-day report, including dogfood, is visible in the
  sidebar.
- **Shared rendering**: niucode's SSE, approvals with diffs, tool cards,
  Markdown, MathML, code highlighting, export, search, and local commands.
- **Boundary**: default build stays niucode-free; `web` and `tui` are separate
  opt-in features.

## Install shape after release

```sh
cargo install niuse --version 0.1.13 --locked
cargo install niuse --version 0.1.13 --features web --locked
cargo install niuse --version 0.1.13 --features tui --locked
```

## Verification

```sh
./scripts/check.sh
cargo +1.88.0 check --workspace --all-targets
cargo +1.88.0 check --features web
```

Last run: default, tui, and web feature checks pass on 1.88; check.sh is green.

## Blockers

- Human approval to publish.
- `niucode 0.1.7` must be published first; `niuse` must require it.
- Version bump and `niucode.rev` pin must be updated in the same release
  commit.
