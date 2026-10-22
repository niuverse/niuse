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

stats="$("$bin" stats --days 1 --json)"
python3 - "$file" "$note" "$stats" <<'PY'
import datetime
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
note = sys.argv[2].strip() or "-"
stats = json.loads(sys.argv[3])
row = "| {date} | {turns} | {proposed} | {accepted} | {note} |".format(
    date=datetime.date.today().isoformat(),
    turns=stats["turns"],
    proposed=stats["proposals"],
    accepted=stats["accepted"],
    note=note.replace("|", "\\|"),
)

lines = path.read_text().splitlines()
header = next(
    (index for index, line in enumerate(lines) if line.strip().startswith("| date | turns")),
    None,
)
if header is None:
    raise SystemExit(f"no dogfood table header in {path}")

end = header + 1
while end < len(lines) and lines[end].lstrip().startswith("|"):
    end += 1

body = [line for line in lines[header + 2 : end] if "YYYY-MM-DD" not in line]
today = row.split(" | ", 1)[0].lstrip("| ").strip()
updated = False
for index, line in enumerate(body):
    if line.lstrip().startswith(f"| {today} |"):
        body[index] = row
        updated = True
        break
if not updated:
    body.append(row)

lines[header + 2 : end] = body
path.write_text("\n".join(lines).rstrip() + "\n")
print(row)
PY
