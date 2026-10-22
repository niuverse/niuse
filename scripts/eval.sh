#!/usr/bin/env bash
# Offline personal-agent eval: prompt/turn regressions plus the MCP notes pack.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

echo "== personal-agent eval =="
cargo test -p niuse --test personal_agent_eval -- --nocapture

echo "== MCP notes pack =="
cargo test -p niuse --test mcp_notes -- --nocapture

echo "eval suite passed"
