#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/docs"
printf '# docs\n' > "$tmp/docs/ok.md"
printf '[ok](docs/ok.md)\n' > "$tmp/README.md"
printf '# agents\n' > "$tmp/AGENTS.md"
python3 "$root/scripts/check-links.py" --root "$tmp" >/dev/null
printf '[bad](docs/missing.md)\n' >> "$tmp/README.md"
if python3 "$root/scripts/check-links.py" --root "$tmp" >/dev/null 2>&1; then
    echo "missing link was accepted" >&2
    exit 1
fi
echo "link checker self-test passed"
