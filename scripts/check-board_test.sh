#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/docs/board" "$tmp/docs/dashboard"

write_project() {
    cat > "$tmp/docs/dashboard/project.json" <<'JSON'
{"milestones":[{"id":"A","status":"done"},{"id":"B","status":"active"},{"id":"C","status":"planned"}]}
JSON
}

card() {
    local id="$1" status="$2" milestone="$3" extra="${4:-}"
    cat > "$tmp/docs/board/$id.md" <<CARD
+++
id = "$id"
title = "card"
status = "$status"
milestone = "$milestone"
size = "S"
updated = "2026-10-22"
$extra
+++

body
CARD
}

write_project
card A-1 done A 'completed = "2026-10-22"'
card B-1 doing B
python3 "$root/scripts/check-board.py" --root "$tmp" >/dev/null

card A-2 ready A
if python3 "$root/scripts/check-board.py" --root "$tmp" >/dev/null 2>&1; then
    echo "open card in done milestone accepted" >&2
    exit 1
fi

rm "$tmp/docs/board/A-2.md"
card C-1 doing C
if python3 "$root/scripts/check-board.py" --root "$tmp" >/dev/null 2>&1; then
    echo "doing card in planned milestone accepted" >&2
    exit 1
fi
rm "$tmp/docs/board/C-1.md"

card B-2 blocked B
if python3 "$root/scripts/check-board.py" --root "$tmp" >/dev/null 2>&1; then
    echo "blocked card without reason accepted" >&2
    exit 1
fi
echo "board checker self-test passed"
