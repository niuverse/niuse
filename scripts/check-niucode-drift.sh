#!/usr/bin/env bash
# Warn when the local niucode checkout no longer matches niucode.rev.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
repo="${NIUCODE_DIR:-$root/../niuverse/niucode}"
pinned="$(cat "$root/niucode.rev")"
local_rev="$(git -C "$repo" rev-parse HEAD)"

if [[ "$pinned" != "$local_rev" ]]; then
  echo "niucode drift: pinned ${pinned:0:7}, local ${local_rev:0:7}" >&2
  echo "run scripts/sync-niucode.sh after reviewing the new revision" >&2
  exit 1
fi

echo "niucode matches pinned ${pinned:0:7}"
