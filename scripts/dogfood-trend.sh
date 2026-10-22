#!/usr/bin/env bash
# Compare the last seven active days with the previous seven.
#
# Exit 0 with "insufficient data" until both windows have at least three active
# days. Exit 1 on an acceptance-rate regression. `--strict` also fails on a
# token-per-turn blow-up or a shrinking active-day count.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
stats_file=""
today="$(date +%F)"
strict=0
while [ "$#" -gt 0 ]; do
    case "$1" in
        --stats) stats_file="$2"; shift 2 ;;
        --today) today="$2"; shift 2 ;;
        --strict) strict=1; shift ;;
        *) echo "usage: dogfood-trend.sh [--stats FILE] [--today YYYY-MM-DD] [--strict]" >&2; exit 2 ;;
    esac
done

if [ -z "$stats_file" ]; then
    bin="${NIUSE_BIN:-niuse}"
    if ! command -v "$bin" >/dev/null 2>&1; then
        if [ -x "$root/target/debug/niuse" ]; then
            bin="$root/target/debug/niuse"
        else
            echo "dogfood trend: no niuse binary and no --stats file; skipped"
            exit 0
        fi
    fi
    stats_file="$(mktemp)"
    trap 'rm -f "$stats_file"' EXIT
    "$bin" stats --days 14 --json > "$stats_file"
fi

python3 - "$stats_file" "$today" "$strict" <<'PY'
import datetime
import json
import sys

path, today_text, strict_text = sys.argv[1], sys.argv[2], sys.argv[3]
strict = strict_text == "1"
today = datetime.date.fromisoformat(today_text)
stats = json.loads(open(path).read())


def window(start, end):
    rows = []
    for row in stats.get("days", []):
        day = datetime.date.fromisoformat(row["date"])
        if start <= day <= end:
            rows.append(row)
    return rows


current = window(today - datetime.timedelta(days=6), today)
previous = window(today - datetime.timedelta(days=13), today - datetime.timedelta(days=7))


def total(rows, key):
    return sum(int(row.get(key, 0)) for row in rows)


def acceptance(rows):
    proposals = total(rows, "proposals")
    return (100.0 * total(rows, "accepted") / proposals) if proposals else None


def tokens_per_turn(rows):
    turns = total(rows, "turns")
    return ((total(rows, "input_tokens") + total(rows, "output_tokens")) / turns) if turns else None


cur_rate, prev_rate = acceptance(current), acceptance(previous)
cur_tokens, prev_tokens = tokens_per_turn(current), tokens_per_turn(previous)
print(
    f"dogfood trend: current {len(current)} days · {total(current, 'turns')} turns · "
    f"acceptance {cur_rate if cur_rate is not None else '-'}% | previous {len(previous)} days · "
    f"{total(previous, 'turns')} turns · acceptance {prev_rate if prev_rate is not None else '-'}%"
)

if len(current) < 3 or len(previous) < 3:
    print("insufficient data: need at least 3 active days in both windows")
    sys.exit(0)

problems = []
warnings = []
if cur_rate is not None and prev_rate is not None:
    if total(current, "proposals") >= 5 and total(previous, "proposals") >= 5 and cur_rate < prev_rate - 20:
        problems.append(f"acceptance dropped {prev_rate:.0f}% -> {cur_rate:.0f}%")
if cur_tokens is not None and prev_tokens is not None:
    if prev_tokens > 0 and cur_tokens > prev_tokens * 2:
        warnings.append(f"tokens/turn rose {prev_tokens:.0f} -> {cur_tokens:.0f}")
if len(current) < len(previous):
    warnings.append(f"active days shrank {len(previous)} -> {len(current)}")

for warning in warnings:
    print(f"warning: {warning}")
if problems:
    for problem in problems:
        print(f"regression: {problem}")
    sys.exit(1)
if strict and warnings:
    print("regression: --strict treats warnings as failures")
    sys.exit(1)
print("dogfood trend: ok")
PY
