//! Offline PTY regression for TUI `/fork` hook notification.

#![cfg(all(unix, feature = "tui"))]

use std::path::Path;
use std::process::Command;

#[test]
fn pty_fork_moves_hooks_to_the_destination() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/tui_fork.py");
    let output = Command::new("python3")
        .arg(script)
        .arg(env!("CARGO_BIN_EXE_niuse"))
        .output()
        .expect("python3 is required for the Unix PTY /fork test");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
