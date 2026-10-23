#!/usr/bin/env bash
# The bundled packs live at the repository root for docs/tests, but the crate
# embeds copies under crates/niuse-cli/assets so cargo package can build from
# the tarball. Keep the embedded copies byte-identical.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
for pack in notes calendar; do
    cmp "$root/packs/$pack/server.py" "$root/crates/niuse-cli/assets/packs/$pack/server.py" \
        || { echo "pack asset drift: $pack/server.py" >&2; exit 1; }
done
echo "pack assets: ok"
