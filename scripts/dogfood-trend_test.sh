#!/usr/bin/env bash
# Deterministic self-test for scripts/dogfood-trend.sh.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

make_json() {
    local file="$1" accepted="$2" turns="$3"
    python3 - "$file" "$accepted" "$turns" <<'PY'
import json, sys
path, accepted, turns = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
days = []
for index in range(7):
    days.append({
        "date": f"2026-10-{9 + index:02d}",
        "turns": turns,
        "proposals": 5,
        "accepted": 4,
        "acceptance_percent": None,
        "input_tokens": 1000 * turns,
        "output_tokens": 100 * turns,
        "cached_tokens": 0,
    })
for index in range(7):
    days.append({
        "date": f"2026-10-{16 + index:02d}",
        "turns": turns,
        "proposals": 5,
        "accepted": accepted,
        "acceptance_percent": None,
        "input_tokens": 1000 * turns,
        "output_tokens": 100 * turns,
        "cached_tokens": 0,
    })
json.dump({"days": days}, open(path, "w"))
PY
}

make_json "$tmp/healthy.json" 4 5
make_json "$tmp/regression.json" 0 5
make_json "$tmp/insufficient.json" 1 1
python3 - "$tmp/insufficient.json" <<'PY'
import json, sys
path = sys.argv[1]
data = json.load(open(path))
data["days"] = [row for row in data["days"] if row["date"] <= "2026-10-13"]
json.dump(data, open(path, "w"))
PY

out="$(bash "$root/scripts/dogfood-trend.sh" --stats "$tmp/healthy.json" --today 2026-10-22)"
[[ "$out" == *"ok"* ]] || { echo "healthy fixture did not pass: $out" >&2; exit 1; }

if bash "$root/scripts/dogfood-trend.sh" --stats "$tmp/regression.json" --today 2026-10-22 >"$tmp/out.txt" 2>&1; then
    echo "regression fixture was not caught" >&2
    exit 1
fi
grep -q "regression" "$tmp/out.txt" || { cat "$tmp/out.txt" >&2; exit 1; }

out="$(bash "$root/scripts/dogfood-trend.sh" --stats "$tmp/insufficient.json" --today 2026-10-22)"
[[ "$out" == *"insufficient"* ]] || { echo "insufficient fixture was not reported: $out" >&2; exit 1; }

echo "dogfood trend self-test passed"
