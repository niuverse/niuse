#!/usr/bin/env bash
# Deterministic self-test for the dogfood log validator.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
header='| date | turns | proposed | accepted | note |
|---|---|---|---|---|'

write() {
    local file="$1"
    shift
    {
        echo "$header"
        printf '%s\n' "$@"
    } > "$file"
}

write "$tmp/partial.md" '| 2026-10-22 | 2 | 1 | 1 | first day |'
out="$("$root/scripts/dogfood-check.sh" --file "$tmp/partial.md" --today 2026-10-22)"
[[ "$out" == *"dogfood days: 1/10"* ]] || { echo "$out" >&2; exit 1; }
if "$root/scripts/dogfood-check.sh" --file "$tmp/partial.md" --today 2026-10-22 --complete >/dev/null 2>&1; then
    echo "partial fixture passed --complete" >&2
    exit 1
fi

{
    echo "$header"
    for day in 01 02 03 04 05 06 07 08 09 10; do
        note='cited m-abcd'
        [[ "$day" == 02 ]] && note='cited m-abcd; disagreed about caching'
        echo "| 2026-10-${day} | 5 | 5 | 3 | $note |"
    done
} > "$tmp/complete.md"
"$root/scripts/dogfood-check.sh" --file "$tmp/complete.md" --today 2026-10-22 --complete >/dev/null
out="$("$root/scripts/dogfood-check.sh" --file "$tmp/complete.md" --today 2026-10-22)"
[[ "$out" == *"status:       complete"* ]] || { echo "$out" >&2; exit 1; }

write "$tmp/duplicate.md" \
    '| 2026-10-21 | 1 | 0 | 0 | - |' \
    '| 2026-10-21 | 1 | 0 | 0 | - |'
if "$root/scripts/dogfood-check.sh" --file "$tmp/duplicate.md" --today 2026-10-22 >/dev/null 2>&1; then
    echo "duplicate accepted" >&2
    exit 1
fi

write "$tmp/over.md" '| 2026-10-21 | 1 | 5 | 6 | bad |'
if "$root/scripts/dogfood-check.sh" --file "$tmp/over.md" --today 2026-10-22 >/dev/null 2>&1; then
    echo "accepted > proposed accepted" >&2
    exit 1
fi

write "$tmp/future.md" '| 2026-12-01 | 1 | 0 | 0 | - |'
if "$root/scripts/dogfood-check.sh" --file "$tmp/future.md" --today 2026-10-22 >/dev/null 2>&1; then
    echo "future day accepted" >&2
    exit 1
fi

echo "dogfood check self-test passed"
