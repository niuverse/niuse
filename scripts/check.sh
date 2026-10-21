#!/usr/bin/env bash
# Every gate the required CI job runs, with failures stopping the script.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

echo "== format =="
cargo fmt --all --check

echo "== clippy =="
cargo clippy --workspace --all-targets -- -D warnings

echo "== tests =="
cargo test --workspace

echo "== dashboard =="
cargo xtask dashboard --check

echo "all checks passed"
