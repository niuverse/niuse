#!/usr/bin/env bash
# Every gate the required CI job runs, with failures stopping the script.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

echo "== format =="
cargo fmt --all --check

echo "== clippy =="
cargo clippy --workspace --all-targets -- -D warnings

echo "== docs =="
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

echo "== tests =="
cargo test --workspace

echo "== tui feature =="
cargo test -p niuse --features tui
cargo clippy -p niuse --features tui --all-targets -- -D warnings

echo "== dogfood =="
cargo build -p niuse
bash scripts/dogfood_test.sh

echo "== package =="
cargo package -p niuse-core --allow-dirty --offline

echo "== dashboard =="
cargo xtask dashboard --check

echo "all checks passed"
