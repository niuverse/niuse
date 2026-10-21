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

## Matrix

| niuse | niucode | status | updated |
|---|---|---|---|
| 0.1.0 | `27b2c55` | current pin; contract tests pass; MSRV declared 1.88 | 2026-10-22 |
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
