#!/usr/bin/env bash
# Pin niuse to a niucode revision and prove the contract still holds.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
repo="${NIUCODE_DIR:-$root/../niucode}"
rev="${1:-$(git -C "$repo" rev-parse HEAD)}"

if ! git -C "$repo" rev-parse --verify "${rev}^{commit}" >/dev/null 2>&1; then
  echo "unknown niucode revision: $rev" >&2
  exit 1
fi
rev="$(git -C "$repo" rev-parse "${rev}^{commit}")"

printf '%s\n' "$rev" > "$root/niucode.rev"
echo "pinned niucode ${rev:0:7}"

cd "$root"
cargo test --workspace
cargo xtask dashboard --check

echo
echo "next: bump the version in docs/compat.md, run \`cargo xtask dashboard\`, and commit niucode.rev"
