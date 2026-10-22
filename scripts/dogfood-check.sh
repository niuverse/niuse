#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec python3 "$root/scripts/dogfood-check.py" --file "$root/docs/dogfood.md" "$@"
