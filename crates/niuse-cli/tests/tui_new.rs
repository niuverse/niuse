//! Offline PTY regression for TUI `/new` isolation.

#![cfg(all(unix, feature = "tui"))]

use std::path::Path;
use std::process::Command;

fn run(mode: &str) {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/tui_new.py");
    let output = Command::new("python3")
        .arg(script)
        .arg(env!("CARGO_BIN_EXE_niuse"))
        .arg(mode)
        .output()
        .expect("python3 is required for the Unix PTY session test");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn pty_new_archives_and_clears_context() {
    run("new");
}

#[test]
fn pty_reset_archives_and_clears_context() {
    run("reset");
}
