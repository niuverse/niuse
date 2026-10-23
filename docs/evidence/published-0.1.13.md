+++
id = "published-0.1.13"
title = "Published · niucode 0.1.7 / niuse 0.1.13"
kind = "doc"
status = "done"
order = 32
summary = "Both releases are on crates.io; clean installs report niucode 0.1.7 and niuse 0.1.13, including the web/tui feature build."
+++

## Published

Date: 2026-10-23, with explicit human approval.

- niucode workspace `0.1.7`: core, auth, provider, tools, extension, mcp, agent,
  and CLI.
- niuse workspace `0.1.13`: `niuse-core` and `niuse`, requiring niucode `0.1.7`.

Registry index confirmed:

```text
niucode 0.1.7
niuse   0.1.13
```

## Install verification

```sh
cargo install niuse --version 0.1.13 --locked --force
# niuse 0.1.13

cargo install niucode --version 0.1.7 --locked --force --root ~/.local
# niucode 0.1.7

cargo install niuse --version 0.1.13 --features web,tui --locked --force
# niuse 0.1.13 with web and tui
```

`niuse --help` lists `web` and `tui`; `niuse status` reports version `0.1.13`
against the local store.

## Notes

- A transitive Windows-only `clipboard-win` lock entry is yanked; Linux installs
  and verification succeeded, and the release was not blocked.
- The default published `niuse` binary remains niucode-free; the local
  `web,tui` install is an explicit opt-in build so the user can try both.
- Dogfood remains 2/10 real days; this release was explicitly approved and did
  not manufacture days.
