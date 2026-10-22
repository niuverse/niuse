#!/usr/bin/env bash
# Offline personal-agent eval: prompt/turn regressions plus the MCP notes pack.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

echo "== personal-agent eval =="
cargo test -p niuse --test personal_agent_eval -- --nocapture

echo "== evidence report =="
cargo test -p niuse --bin niuse evidence::tests -- --nocapture
cargo test -p niuse --test cli evidence -- --nocapture

echo "== recorded verification =="
cargo test -p niuse --test cli verify -- --nocapture

echo "== plan verification gate =="
cargo test -p niuse --test cli plan_steps -- --nocapture

echo "== prompt budget =="
cargo test -p niuse --test prompt_budget -- --nocapture

echo "== cost signals =="
cargo test -p niuse --test cli stats_json_reports -- --nocapture

echo "== agent recall =="
cargo test -p niuse --bin niuse memory_tool::tests -- --nocapture

echo "== MCP notes pack =="
cargo test -p niuse --test mcp_notes -- --nocapture

echo "== MCP calendar pack =="
cargo test -p niuse --test mcp_calendar -- --nocapture

if [[ -d "$root/../niucode" ]]; then
  echo "== MCP client boundaries =="
  (cd "$root/../niucode" && cargo test -p niucode-mcp --tests -- --nocapture)
  echo "== MCP agent dispatch =="
  (cd "$root/../niucode" && cargo test -p niucode-agent --test mcp -- --nocapture)
fi

echo "eval suite passed"
