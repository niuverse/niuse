//! Offline PTY regression for TUI mutating-MCP approval.

#![cfg(all(unix, feature = "tui"))]

use std::path::Path;
use std::process::Command;

fn run(mode: &str) {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/tui_mcp_write.py");
    let output = Command::new("python3")
        .arg(script)
        .arg(env!("CARGO_BIN_EXE_niuse"))
        .arg(mode)
        .output()
        .expect("python3 is required for the Unix PTY write-MCP test");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn pty_write_mcp_deny_writes_nothing() {
    run("deny");
}

#[test]
fn pty_write_mcp_allow_writes_once() {
    run("allow");
}
