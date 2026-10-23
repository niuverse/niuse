+++
id = "feature-dry-run"
title = "Release feature dry run · web/tui"
kind = "doc"
status = "done"
order = 25
summary = "tui packages and verifies against niucode 0.1.6; web fails as expected because niucode::web is not in the published 0.1.6 crate."
+++

## Context

Date: 2026-10-23. HEAD before this note: `91a27db`. Version `0.1.12`.
Pinned niucode: `6ea13f9` (workspace `0.1.6`). No upload.

## Commands

```sh
cargo publish --dry-run -p niuse --features tui --allow-dirty
cargo publish --dry-run -p niuse --features web --allow-dirty
```

## Result

| Features | Package | Verify | Reason |
| --- | --- | --- | --- |
| `tui` | pass | pass | uses the published 0.1.6 API |
| `web` | pass | **fail** | `niucode::web` is absent from the registry 0.1.6 |

Exact failure:

```text
error[E0433]: cannot find `web` in `niucode`
  --> src/main.rs:835:34
```

## Interpretation

This is the expected release dependency, not a source regression:

- `niuse 0.1.12` on crates.io depends on `niucode 0.1.6`.
- The web surface was added after 0.1.6 and is only available in the sibling
  checkout.
- The current checkout passes `--features web` locally.
- Publishing `niuse 0.1.13` with `web` requires `niucode 0.1.7` first.

## Release gate

1. Publish niucode 0.1.7 (with `niucode::web` and the session APIs).
2. Bump niuse to 0.1.13 and require niucode 0.1.7.
3. Re-run both feature dry runs; `web` must pass.
4. Test `cargo install` and the tokenized web startup.
