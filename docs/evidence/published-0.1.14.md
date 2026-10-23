+++
id = "published-0114"
title = "Published 0.1.14 evidence"
kind = "evidence"
status = "living"
order = 93
summary = "niuse-core/niuse 0.1.14 are on crates.io; default and web+tui clean installs report 0.1.14."
+++

## Registry

Checked 2026-10-24:

```text
niuse-core 0.1.14
niuse 0.1.14
```

Both require the published `niucode 0.1.8` crates.

## Clean installs

Default:

```sh
cargo install niuse --version 0.1.14 --locked --root /tmp/niuse-install-0114
/tmp/niuse-install-0114/bin/niuse --version
```

Result:

```text
niuse 0.1.14
```

Opt-in web+tui:

```sh
cargo install niuse --version 0.1.14 --features web,tui --locked \
  --root /tmp/niuse-install-0114-full
```

Result:

```text
niuse 0.1.14
```

## Kernel install

```sh
cargo install niucode --version 0.1.8 --locked --root /tmp/niucode-install-018
/tmp/niucode-install-018/bin/niucode --version
```

Result:

```text
niucode 0.1.8
```

## Source

- published code commit: `b28fd9d`
- docs/evidence commit: the commit containing this file

No crate was yanked.
