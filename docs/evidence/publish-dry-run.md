+++
id = "publish-dry-run"
title = "Publish dry run · niuse 0.1.12"
kind = "doc"
status = "done"
order = 24
summary = "Both publishable niuse crates package and verify under cargo publish --dry-run; the opt-in web feature is not covered by package verification."
+++

## Context

Date: 2026-10-23. HEAD: `300dd95`. Workspace version: `0.1.12`.
Pinned niucode: `6ea13f9` (workspace `0.1.6`). No upload.

## Command

```sh
cargo publish --dry-run -p niuse-core --allow-dirty
cargo publish --dry-run -p niuse --allow-dirty
```

## Result

| Crate | Package | Verify | Upload |
| --- | --- | --- | --- |
| niuse-core | pass | pass | aborted (dry run) |
| niuse | pass | pass | aborted (dry run) |

## What this proves

- The default `niuse` package contains the files Cargo expects.
- It verifies against the currently published `niucode` 0.1.6 because the
  default features do not link niucode's TUI/web code.
- `niuse-core` packages independently.

## What this does not prove

- It does not compile the opt-in `web` or `tui` features.
- It does not prove the next `niucode` release resolves correctly.
- It does not upload, tag, or bump.
- The package still depends on `niucode = 0.1.6`; 0.1.13 will require 0.1.7.

## Release implication

After publishing niucode 0.1.7, rerun the dry run with the new requirement and
test `--features web`/`--features tui` explicitly.
