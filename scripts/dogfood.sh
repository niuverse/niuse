#!/usr/bin/env bash
# Append (or replace) today's row in the ten-day dogfood table.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
file="$root/docs/dogfood.md"
note=""

while [ "$#" -gt 0 ]; do
    case "$1" in
        --file)
            file="$2"
            shift 2
            ;;
        *)
            if [ -n "$note" ]; then
                note="$note $1"
            else
                note="$1"
            fi
            shift
            ;;
    esac
done

bin="${NIUSE_BIN:-niuse}"
if ! command -v "$bin" >/dev/null 2>&1; then
    if [ -x "$root/target/debug/niuse" ]; then
        bin="$root/target/debug/niuse"
    else
        echo "niuse not found; set NIUSE_BIN or build ./target/debug/niuse" >&2
        exit 1
    fi
fi

if [ -n "$note" ]; then
    exec "$bin" dogfood --file "$file" --note "$note"
fi
exec "$bin" dogfood --file "$file"
