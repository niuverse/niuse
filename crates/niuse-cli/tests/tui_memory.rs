//! Offline PTY regression for the TUI memory-candidate prompt.

#![cfg(all(unix, feature = "tui"))]

use std::path::Path;
use std::process::Command;

fn run(mode: &str) {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/tui_memory.py");
    let output = Command::new("python3")
        .arg(script)
        .arg(env!("CARGO_BIN_EXE_niuse"))
        .arg(mode)
        .output()
        .expect("python3 is required for the Unix PTY memory test");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn pty_memory_accept_stores_the_candidate() {
    run("accept");
}

#[test]
fn pty_memory_discard_does_not_store_the_candidate() {
    run("discard");
}
