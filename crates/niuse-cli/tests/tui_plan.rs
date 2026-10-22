//! Offline PTY regression for the TUI plan ledger and tagged verification.

#![cfg(all(unix, feature = "tui"))]

use std::path::Path;
use std::process::Command;

#[test]
fn pty_tui_closes_a_step_only_after_tagged_verification() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("tui_plan.py");
    let output = Command::new("python3")
        .arg(script)
        .arg(env!("CARGO_BIN_EXE_niuse"))
        .output()
        .expect("python3 is required for the Unix PTY plan test");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
