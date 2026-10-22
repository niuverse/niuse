//! Offline PTY regression for the `niuse tui` approval path.
//!
//! The scenario is driven by the replay provider, so no API key or network is
//! involved. Python 3 supplies the PTY, the same prerequisite the niucode TUI
//! acceptance tests use.

#![cfg(all(unix, feature = "tui"))]

use std::path::Path;
use std::process::Command;

#[test]
fn pty_approval_uses_configured_keys() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("tui_approval.py");
    let output = Command::new("python3")
        .arg(script)
        .arg(env!("CARGO_BIN_EXE_niuse"))
        .output()
        .expect("python3 is required for the Unix PTY approval test");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
