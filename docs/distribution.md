+++
id = "distribution"
title = "Distribution notes"
kind = "doc"
status = "living"
order = 50
summary = "What installs today, why crates.io is blocked by path dependencies, and the decisions still open: dependency strategy, names, and license."
+++

## What works today

```sh
cargo install --git https://github.com/niuverse/niuse
```

This is also the path for people who only want the binary. It bypasses crates.io entirely.

## Why crates.io is blocked

`cargo package -p niuse` fails:

```text
error: all dependencies must have a version requirement specified when packaging.
dependency `niucode-agent` does not specify a version
```

crates.io removes `path` specifications and expects the dependency to exist in the registry. Until that changes, only `niuse-core` is packageable on its own, and it is not useful without the CLI.

## Options

| Option | Cost | When |
|---|---|---|
| Publish the `niucode-*` crates first, then depend on versions | Release coordination between the two repositories | After niucode's embedding surface stabilizes |
| Switch to git dependencies with a pinned revision | Every build fetches from GitHub; local co-development needs a patch override | If clean-checkout builds matter before publishing |
| Keep the CLI on `cargo install --git` | No crates.io presence or docs.rs page | Acceptable while the tool is personal |

The pinned-revision policy in [compat](compat.md) is independent of this choice; it keeps the required CI reproducible either way.

## Names

`niuse`, `niuse-core`, `niuse-cli`, and `niucode` were unclaimed on crates.io, npm, and PyPI when checked on 2026-10-21. Recheck immediately before publishing; registry names are first-come. The binary name is independent of the crate name, so a rename is recoverable.

## License

Not chosen yet. crates.io requires a `license` or `license-file`, so publishing waits on that decision. The workspace `repository` field is set; the package warning now only complains about the missing license.

## Checklist

- [ ] Choose a license and add it to the workspace manifest.
- [ ] Decide the niucode dependency strategy (publish first, git rev, or git-only install).
- [ ] Recheck crate names and reserve them with a real release.
- [ ] Add a release workflow that runs `scripts/check.sh` before publish.
