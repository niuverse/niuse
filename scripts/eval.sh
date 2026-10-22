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

echo "== MCP init =="
cargo test -p niuse --test cli mcp_init -- --nocapture

echo "== MCP init merge =="
cargo test -p niuse --test cli mcp_init_merges -- --nocapture

echo "== TUI MCP PTY =="
cargo test -p niuse --features tui --test tui_mcp -- --nocapture

echo "== combined personal workflow =="
cargo test -p niuse --test personal_workflow -- --nocapture

echo "== TUI plan/verify =="
cargo test -p niuse --features tui --bin niuse read_only_commands -- --nocapture

echo "== multi-turn plan execution =="
cargo test -p niuse --test plan_execution -- --nocapture

echo "== dogfood row ==
cargo test -p niuse --test cli dogfood_writes -- --nocapture

echo "== TUI plan/verify PTY =="
cargo test -p niuse --features tui --test tui_plan -- --nocapture

if [[ -d "$root/../niucode" ]]; then
  echo "== MCP client boundaries =="
  (cd "$root/../niucode" && cargo test -p niucode-mcp --tests -- --nocapture)
  echo "== MCP agent dispatch =="
  (cd "$root/../niucode" && cargo test -p niucode-agent --test mcp -- --nocapture)
fi

echo "eval suite passed"
