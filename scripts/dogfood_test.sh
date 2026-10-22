#!/usr/bin/env bash
# End-to-end check for scripts/dogfood.sh: one row, then replace, no duplicate.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cp "$root/docs/dogfood.md" "$tmp/dogfood.md"
mkdir -p "$tmp/home"
stamp="$(python3 -c 'import time; print(int(time.time() * 1000))')"
printf '%s\tturn\tok\n' "$stamp" > "$tmp/home/metrics.jsonl"
today="$(date +%F)"

NIUSE_BIN="$root/target/debug/niuse" NIUSE_HOME="$tmp/home" \
    "$root/scripts/dogfood.sh" --file "$tmp/dogfood.md" "first note" >/dev/null
grep -q "| $today | 1 | 0 | 0 | first note |" "$tmp/dogfood.md"

NIUSE_BIN="$root/target/debug/niuse" NIUSE_HOME="$tmp/home" \
    "$root/scripts/dogfood.sh" --file "$tmp/dogfood.md" "second note" >/dev/null
test "$(grep -c "| $today |" "$tmp/dogfood.md")" -eq 1
grep -q "| $today | 1 | 0 | 0 | second note |" "$tmp/dogfood.md"

echo "dogfood script ok"
