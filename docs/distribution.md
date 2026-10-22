+++
id = "distribution"
title = "Distribution notes"
kind = "doc"
status = "living"
order = 50
summary = "A two-checkout build works; cargo install --git and crates.io do not, because niucode's path dependencies sit outside this repository. The dependency strategy and license are still open."
+++

## What works today

niuse builds from a workspace containing both checkouts at the expected relative paths:

```text
workspace/
  niuse/          this repository
  niuverse/
    niucode/      the pinned checkout
```

```sh
git clone https://github.com/niuverse/niuse
git clone https://github.com/niuverse/niucode niuverse/niucode
cd niuse
cargo build --release
# or install the binary from the checkout
cargo install --path crates/niuse-cli --locked
```

Verified on 2026-10-22: `cargo install --path crates/niuse-cli --root /tmp/niuse-install --locked --debug` produced a working `niuse 0.1.0`.

## Why `cargo install --git` does not work

`cargo install --git https://github.com/niuverse/niuse` clones only niuse. The dependencies `niucode-agent`, `niucode-core`, and the other `niucode-*` crates are declared as `path = "../niuverse/niucode/crates/..."`, so a clean clone has no such directory and Cargo cannot fetch it.

Verified on 2026-10-22 by cloning the repository into `/tmp` and running `cargo metadata`:

```text
error: failed to load manifest for workspace member `.../crates/niuse-cli`
Caused by: failed to load manifest for dependency `niucode-agent`
```

The README quick start therefore requires a local checkout of `../niuverse/niucode`; that requirement is real, not a temporary convenience.

## Why crates.io is blocked

`cargo package -p niuse` fails:

```text
error: all dependencies must have a version requirement specified when packaging.
dependency `niucode-agent` does not specify a version
```

crates.io removes `path` specifications and expects the dependency to exist in the registry. Until that changes, only `niuse-core` is packageable on its own, and it is not useful without the CLI.

## Chosen path (2026-10-22)

Publish the `niucode-*` crates first at **0.1.0**, then consume them by version. The path dependencies in niuse already carry `version = "0.1.0"` beside `path`, so local development keeps the sibling checkout while packaging can resolve registry versions once the upload exists.

Remaining before the upload: choose a license, run `cargo login`, and publish in the order documented in niucode's `docs/release.md`. Until then, `cargo package -p niuse` still cannot verify against the registry.

## Options

| Option | Cost | When |
|---|---|---|
| Publish the `niucode-*` crates first, then depend on versions | Release coordination between the two repositories | After niucode's embedding surface stabilizes |
| Convert the dependency spec to git with a pinned revision | Every clean build fetches from GitHub; local co-development needs `[patch]` overrides | If clean-checkout installs matter before publishing |
| Keep the two-checkout build | No crates.io presence, no docs.rs page, no one-command install | Acceptable while the tool is personal |

A `cargo install --git` line is **not** among the options: it cannot resolve the path dependencies. Switching to git dependencies is a different change; it edits `Cargo.toml` and keeps the pin in the revision.

The pinned-revision policy in [compat](compat.md) is independent of this choice; it keeps the required CI reproducible either way.

## Names

`niuse`, `niuse-core`, `niuse-cli`, and `niucode` were unclaimed on crates.io, npm, and PyPI when checked on 2026-10-21. Recheck immediately before publishing; registry names are first-come. The binary name is independent of the crate name, so a rename is recoverable.

## License

`MIT OR Apache-2.0`, the usual Rust dual license. `LICENSE-MIT` and `LICENSE-APACHE` are at the repository root and `license.workspace` is inherited by every crate. crates.io can now accept the packages.

## Checklist

- [ ] Choose a license and add it to the workspace manifest.
- [x] Choose publishing the niucode crates first at 0.1.0; until the upload exists, clean-checkout installs still need the sibling checkout.
- [ ] Add `CARGO_REGISTRY_TOKEN` / `cargo login` and run the release workflow.
- [ ] Recheck crate names and reserve them with a real release.
- [ ] Add a release workflow that runs `scripts/check.sh` before publish.
