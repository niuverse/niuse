//! Offline PTY regression for TUI `/resume` hook notification.

#![cfg(all(unix, feature = "tui"))]

use std::path::Path;
use std::process::Command;

#[test]
fn pty_resume_moves_hooks_to_the_resumed_session() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/tui_resume.py");
    let output = Command::new("python3")
        .arg(script)
        .arg(env!("CARGO_BIN_EXE_niuse"))
        .output()
        .expect("python3 is required for the Unix PTY /resume test");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
