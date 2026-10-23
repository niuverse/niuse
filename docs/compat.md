+++
id = "compat"
title = "niucode compatibility"
kind = "doc"
status = "living"
order = 30
summary = "Pinned niucode revision, the weekly bump ritual, and the nightly check against main."
+++

## Policy

- niuse pins niucode in `niucode.rev`; required CI and Pages check out exactly that commit.
- The pin moves at most once a week, through `scripts/sync-niucode.sh`.
- A nightly workflow runs the same tests against niucode `main`. Failures are an early signal, not a pull-request gate.
- `scripts/check-niucode-drift.sh` reports when the local sibling checkout no longer matches the pin.

The front-end seam is proposed upstream in [ADR-0032](https://github.com/niuverse/niucode/blob/main/docs/adr/0032-embedding-library-surface.md); this document tracks the call surface niuse uses today.

The contract is deliberately small: `run_with_history`, `AgentConfig`, `project_history`, `SessionLog` and `Event`, the provider trait, credentials, sandbox and approval types, per-turn `working_dir` on one session, and approval denial carrying a structured preview. `crates/niuse-cli/tests/niucode_contract.rs` exercises them offline with `ScriptedProvider`, so the check costs no tokens.

One read-only exception: `niuse projects` reads niucode's workspace metadata at `~/.niucode/projects/*/project.toml` (schema 1: `cwd`, `last_seen`) and counts `sessions/*.jsonl`. This is a display view, not a second index; if niucode changes the layout, update `crates/niuse-cli/src/projects.rs` and its test.

All niuse paths mount niucode's read-only MCP host: `niuse tui` inherits it, and the REPL/`exec` call `niucode-mcp::McpHost` through `driver::mount_mcp` from `NIUSE_MCP_CONFIG` or `~/.niucode/mcp.toml`. A broken manifest disables MCP for that run with a warning.

## Matrix

| niuse | niucode | status | updated |
|---|---|---|---|
| 0.1.12 | `1387acb` | current pin; web context sidebar | 2026-10-23 |
| 0.1.12 | `39f68cc` | previous pin; niuse web prompt | 2026-10-23 |
| 0.1.12 | `bed82c4` | previous pin; web MathML | 2026-10-23 |
| 0.1.12 | `a01ef09` | previous pin; web Markdown | 2026-10-23 |
| 0.1.12 | `ec258d1` | previous pin; browser approvals | 2026-10-23 |
| 0.1.12 | `cb2c73f` | previous pin; streaming web turns | 2026-10-23 |
| 0.1.12 | `257dde2` | previous pin; browser prompt | 2026-10-23 |
| 0.1.12 | `ee0e1c5` | previous pin; live web SSE tail | 2026-10-23 |
| 0.1.12 | `0364daa` | previous pin; local web viewer | 2026-10-23 |
| 0.1.12 | `1ff6738` | previous pin; streaming caret + tool timer | 2026-10-23 |
| 0.1.12 | `6669376` | previous pin; display math | 2026-10-23 |
| 0.1.12 | `7c6ee9d` | previous pin; code fence chrome | 2026-10-23 |
| 0.1.12 | `0dd472c` | previous pin; lists/tasks/quotes | 2026-10-23 |
| 0.1.12 | `c389bf9` | previous pin; math fallback | 2026-10-23 |
| 0.1.12 | `027d038` | previous pin; Markdown/table TUI rendering | 2026-10-23 |
| 0.1.12 | `8d31777` | previous pin; niucode 0.1.6 release docs | 2026-10-23 |
| 0.1.12 | `64a23b9` | previous pin; niucode 0.1.6 release API | 2026-10-23 |
| 0.1.12 | `56ae465` | previous pin; MCP mount audit + subagent isolation | 2026-10-23 |
| 0.1.12 | `ab50b75` | previous pin; ADR-0034 write path | 2026-10-23 |
| 0.1.11 | `2b278fe` | previous pin; TUI resume/fork hook notification | 2026-10-22 |
| 0.1.11 | `323a158` | previous pin; TUI new-session seam | 2026-10-22 |
| 0.1.11 | `0da0865` | previous pin; TUI `/new` alias | 2026-10-22 |
| 0.1.11 | `c0e8c38` | previous pin; step-tagged verification | 2026-10-22 |
| 0.1.11 | `6dc0f13` | previous pin; verification events | 2026-10-22 |
| 0.1.11 | `f38df22` | previous pin; read-only MCP mounting in exec/TUI | 2026-10-22 |
| 0.1.11 | `eaece28` | previous pin; read-only embedder tools | 2026-10-22 |
| 0.1.11 | `2d12360` | previous pin; configurable approval keys | 2026-10-22 |
| 0.1.5 | `76fec7b` | previous pin; command registration released | 2026-10-22 |
| 0.1.4 | `9ffed27` | previous pin; post-turn prompts released | 2026-10-22 |
| 0.1.3 | `54df537` | previous pin; filter and command hooks released | 2026-10-22 |
| 0.1.2 | `3ec224d` | previous pin; filter_text on main | 2026-10-22 |
| 0.1.2 | `40d6cd8` | previous pin; full TUI hooks released | 2026-10-22 |
| 0.1.1 | `a4420cb` | previous pin; full TUI hooks on main | 2026-10-22 |
| 0.1.1 | `21148ca` | previous pin; 0.1.1 published | 2026-10-22 |
| 0.1.0 | `5d94b0d` | previous pin; CJK spacing and TUI prompt hook | 2026-10-22 |
| 0.1.0 | `cd311a8` | previous pin; 0.1.0 published and install verified | 2026-10-22 |
| 0.1.0 | `28f867e` | previous pin; release retry and email preflight | 2026-10-22 |
| 0.1.0 | `452f92a` | previous pin; hook test works without bwrap | 2026-10-22 |
| 0.1.0 | `3d635cb` | previous pin; exec hooks wired | 2026-10-22 |
| 0.1.0 | `fb68b31` | previous pin; changelog corrected | 2026-10-22 |
| 0.1.0 | `aac20a1` | previous pin; ADR-0032 library target landed | 2026-10-22 |
| 0.1.0 | `d6d69f0` | previous pin; 0.1.0 changelog added | 2026-10-22 |
| 0.1.0 | `3508a94` | previous pin; release workflow tests before publish | 2026-10-22 |
| 0.1.0 | `1753d05` | previous pin; release workflow added | 2026-10-22 |
| 0.1.0 | `b7e75b0` | previous pin; crates prepared for 0.1.0 publishing | 2026-10-22 |
| 0.1.0 | `f3711b1` | previous pin; session files created 0600 | 2026-10-22 |
| 0.1.0 | `f65ca7a` | previous pin; forward-compat tests | 2026-10-22 |
| 0.1.0 | `27b2c55` | previous pin; MSRV declared 1.88 | 2026-10-22 |
| 0.1.0 | `eabc0c4` | previous pin; ADR-0032 sketch added | 2026-10-22 |
| 0.1.0 | `6fd40ca` | previous pin; contract tests pass | 2026-10-22 |
| 0.1.0 | `95f5303` | previous pin; contract tests pass | 2026-10-21 |
| 0.1.0 | `b5f66ba` | previous pin; contract tests pass | 2026-10-21 |

## Weekly ritual

```sh
./scripts/sync-niucode.sh <rev>   # writes niucode.rev, runs tests + dashboard check
cargo xtask dashboard             # refresh the board
# update the matrix above, commit niucode.rev and the generated board
```

If the nightly job is red, read the contract test failure first. A break there usually means one of two things: an API niuse uses changed shape, or the behaviour behind it changed. Fix upstream when the change is generic; adapt niuse when it is product-specific.
