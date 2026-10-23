#!/usr/bin/env bash
# Read-only release preflight across the pinned niucode and this niuse tree.
# Publishing is intentionally NOT part of this script.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
niucode="${NIUCODE_DIR:-$root/../niucode}"

if [[ ! -d "$niucode/.git" ]]; then
  echo "niucode checkout not found at $niucode (set NIUCODE_DIR)" >&2
  exit 1
fi

echo "== repositories =="
for repo in "$niucode" "$root"; do
  name="$(basename "$repo")"
  if [[ -n "$(git -C "$repo" status --porcelain)" ]]; then
    echo "$name has uncommitted changes" >&2
    exit 1
  fi
  echo "$name clean at $(git -C "$repo" rev-parse --short HEAD)"
done

echo "== pin =="
pinned="$(cat "$root/niucode.rev")"
head="$(git -C "$niucode" rev-parse HEAD)"
if [[ "$pinned" != "$head" ]]; then
  echo "niucode.rev $pinned does not match checkout $head" >&2
  exit 1
fi
echo "pin matches $head"

echo "== niucode MSRV =="
(cd "$niucode" && cargo +1.88.0 check --workspace --all-targets)

echo "== niucode tests =="
(cd "$niucode" && cargo test --workspace)

echo "== niucode clippy =="
(cd "$niucode" && cargo clippy --workspace --all-targets -- -D warnings)

echo "== niucode package =="
(cd "$niucode" && cargo package -p niucode --allow-dirty)

echo "== niucode dashboard =="
(cd "$niucode" && cargo xtask dashboard --check)

echo "== niuse MSRV =="
(cd "$root" && cargo +1.88.0 check --workspace --all-targets)
(cd "$root" && cargo +1.88.0 check --features web)
(cd "$root" && cargo +1.88.0 check --features tui)

echo "== niuse gates =="
(cd "$root" && ./scripts/check.sh)

echo "== drift =="
(cd "$root" && ./scripts/check-niucode-drift.sh)

echo "preflight passed; publishing is still a separate human step"
