//! Setup checks for `niuse doctor`.
//!
//! Kept separate from the store commands: it is the only command that inspects
//! credentials, the sandbox backend, and the pinned niucode checkout.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Result};
use niucode_auth::AuthStore;
use niucode_provider::Preset;
use niuse_core::{Date, Store};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Level {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize)]
struct Check {
    level: Level,
    label: String,
    detail: String,
}

#[derive(Debug, Default)]
struct Report {
    checks: Vec<Check>,
    failures: usize,
    warnings: usize,
}

impl Report {
    fn record(&mut self, level: Level, label: &str, detail: impl Into<String>) {
        self.checks.push(Check {
            level,
            label: label.to_string(),
            detail: detail.into(),
        });
        match level {
            Level::Ok => {}
            Level::Warn => self.warnings += 1,
            Level::Fail => self.failures += 1,
        }
    }

    fn print(&self) {
        for check in &self.checks {
            let tag = match check.level {
                Level::Ok => "ok  ",
                Level::Warn => "warn",
                Level::Fail => "FAIL",
            };
            println!("[{tag}] {:<11} {}", check.label, check.detail);
        }
    }
}

pub fn doctor(store: &Store, preset: &str, json: bool) -> Result<()> {
    let mut report = Report::default();

    match fs::metadata(store.root()) {
        Ok(metadata) => {
            let mode = mode_of(&metadata);
            let writable = write_probe(store.root());
            let level = if writable { Level::Ok } else { Level::Fail };
            report.record(
                level,
                "home",
                format!(
                    "{} ({mode}, {})",
                    store.root().display(),
                    if writable { "writable" } else { "not writable" }
                ),
            );
        }
        Err(error) => report.record(
            Level::Fail,
            "home",
            format!("{}: {error}", store.root().display()),
        ),
    }

    match store.identity()? {
        Some(text) if !text.trim().is_empty() => {
            report.record(Level::Ok, "identity", format!("set ({} bytes)", text.len()))
        }
        _ => report.record(
            Level::Warn,
            "identity",
            "missing; run `niuse identity init`",
        ),
    }

    match store.user()? {
        Some(text) if !text.trim().is_empty() => {
            report.record(Level::Ok, "user", format!("set ({} bytes)", text.len()))
        }
        _ => report.record(
            Level::Warn,
            "user",
            "missing; run `niuse bootstrap --profile` or `niuse user edit`",
        ),
    }

    let today = Date::today();
    let memories = store.memories()?;
    report.record(
        Level::Ok,
        "memory",
        format!(
            "{} entries ({} stale, {} expired)",
            memories.len(),
            memories
                .iter()
                .filter(|entry| entry.is_stale(today) && !entry.is_expired(today))
                .count(),
            memories
                .iter()
                .filter(|entry| entry.is_expired(today))
                .count()
        ),
    );

    let thread = store.session_path();
    if thread.exists() {
        match niucode_core::load(&thread) {
            Ok(loaded) => report.record(
                Level::Ok,
                "thread",
                format!("{} ({} events)", thread.display(), loaded.events.len()),
            ),
            Err(error) => report.record(Level::Warn, "thread", format!("unreadable: {error:#}")),
        }
    } else {
        report.record(Level::Ok, "thread", "new; will be created on first run");
    }

    private_path(
        &mut report,
        "history",
        &store.history_path(),
        0o600,
        "prompt history",
    );
    private_path(
        &mut report,
        "backups",
        &store.root().join("backups"),
        0o700,
        "prune backups",
    );

    match store.content_issues() {
        Ok(issues) if issues.is_empty() => {
            report.record(Level::Ok, "parse", "all stored content parsed")
        }
        Ok(issues) => report.record(Level::Warn, "parse", issues.join("; ")),
        Err(error) => report.record(Level::Fail, "parse", format!("{error:#}")),
    }

    let metrics = store.metrics()?;
    report.record(
        Level::Ok,
        "metrics",
        format!("{} records", metrics.lines().count()),
    );

    match crate::settings::load(store) {
        Ok(_) if store.config_path().exists() => report.record(
            Level::Ok,
            "config",
            format!("{} (valid)", store.config_path().display()),
        ),
        Ok(_) => report.record(Level::Ok, "config", "defaults (no config.toml)"),
        Err(error) => report.record(Level::Fail, "config", format!("{error:#}")),
    }

    match Preset::parse(preset) {
        Some(preset) => match AuthStore::new() {
            Ok(auth) => match auth.resolve(preset.as_str()) {
                Ok(Some(resolved)) => report.record(
                    Level::Ok,
                    "credential",
                    format!("{} via {}", preset.as_str(), resolved.source),
                ),
                Ok(None) => report.record(
                    Level::Fail,
                    "credential",
                    format!("missing; run `niucode auth set {}`", preset.as_str()),
                ),
                Err(error) => report.record(Level::Fail, "credential", format!("{error}")),
            },
            Err(error) => report.record(Level::Fail, "credential", format!("{error}")),
        },
        None => report.record(
            Level::Fail,
            "credential",
            format!("unknown provider {preset:?}"),
        ),
    }

    if niucode_tools::sandbox_available() {
        report.record(Level::Ok, "sandbox", "bwrap available");
    } else {
        report.record(
            Level::Warn,
            "sandbox",
            "bwrap missing; approvals fall back to per-action prompts",
        );
    }

    match pin_status() {
        Some(PinStatus::Match(rev)) => report.record(
            Level::Ok,
            "pin",
            format!("{rev} matches the local checkout"),
        ),
        Some(PinStatus::Drift { pinned, local }) => report.record(
            Level::Warn,
            "pin",
            format!("pinned {pinned}, local {local}; run scripts/sync-niucode.sh"),
        ),
        Some(PinStatus::Missing) => report.record(
            Level::Warn,
            "pin",
            "niucode.rev found, sibling checkout missing",
        ),
        None => report.record(Level::Ok, "pin", "no niucode.rev (installed binary)"),
    }

    if json {
        let payload = serde_json::json!({
            "checks": report.checks,
            "failures": report.failures,
            "warnings": report.warnings,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        report.print();
        println!(
            "\ndoctor: {} failure(s), {} warning(s)",
            report.failures, report.warnings
        );
    }
    if report.failures > 0 {
        bail!("doctor found {} failure(s)", report.failures);
    }
    Ok(())
}

enum PinStatus {
    Match(String),
    Drift { pinned: String, local: String },
    Missing,
}

fn pin_status() -> Option<PinStatus> {
    let root = repo_root()?;
    let pinned = fs::read_to_string(root.join("niucode.rev")).ok()?;
    let pinned = pinned.trim().to_string();
    if pinned.is_empty() {
        return None;
    }
    let Some(parent) = root.parent() else {
        return Some(PinStatus::Missing);
    };
    let checkout = parent.join("niuverse").join("niucode");
    if !checkout.is_dir() {
        return Some(PinStatus::Missing);
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(&checkout)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return Some(PinStatus::Missing);
    }
    let local = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let pinned_short = pinned.chars().take(local.len()).collect::<String>();
    if pinned_short == local {
        Some(PinStatus::Match(pinned_short))
    } else {
        Some(PinStatus::Drift {
            pinned: pinned_short,
            local,
        })
    }
}

fn repo_root() -> Option<PathBuf> {
    let mut directory = std::env::current_dir().ok()?;
    loop {
        if directory.join("niucode.rev").is_file() {
            return Some(directory);
        }
        if !directory.pop() {
            return None;
        }
    }
}

fn write_probe(directory: &std::path::Path) -> bool {
    let probe = directory.join(".doctor-write-test");
    match fs::write(&probe, b"probe") {
        Ok(()) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

#[cfg(unix)]
fn private_path(report: &mut Report, label: &str, path: &Path, expected: u32, what: &str) {
    match fs::metadata(path) {
        Ok(metadata) => {
            let mode = mode_bits(&metadata);
            let mode_text = mode_of(&metadata);
            if is_private_mode(mode) {
                report.record(
                    Level::Ok,
                    label,
                    format!("{} ({mode_text})", path.display()),
                );
            } else {
                report.record(
                    Level::Warn,
                    label,
                    format!(
                        "{} is group/world readable ({mode_text}); {what} should be {expected:04o}, run chmod {expected:o}",
                        path.display()
                    ),
                );
            }
        }
        Err(_) => report.record(Level::Ok, label, "not created yet"),
    }
}

fn is_private_mode(mode: u32) -> bool {
    mode & 0o077 == 0
}

#[cfg(unix)]
fn mode_bits(metadata: &fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o777
}

#[cfg(not(unix))]
fn mode_bits(_metadata: &fs::Metadata) -> u32 {
    0
}

fn mode_of(metadata: &fs::Metadata) -> String {
    format!("{:04o}", mode_bits(metadata))
}

#[cfg(not(unix))]
fn mode_of(_metadata: &fs::Metadata) -> String {
    "mode n/a".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_modes_reject_group_and_world_bits() {
        assert!(is_private_mode(0o600));
        assert!(is_private_mode(0o700));
        assert!(is_private_mode(0o400));
        assert!(!is_private_mode(0o640));
        assert!(!is_private_mode(0o604));
        assert!(!is_private_mode(0o755));
    }

    #[test]
    fn report_counts_and_serializes_levels() {
        let mut report = Report::default();
        report.record(Level::Ok, "a", "b");
        report.record(Level::Warn, "c", "d");
        report.record(Level::Fail, "e", "f");
        assert_eq!(report.failures, 1);
        assert_eq!(report.warnings, 1);
        let json = serde_json::to_value(&report.checks).unwrap();
        assert_eq!(json[0]["level"], "ok");
        assert_eq!(json[1]["level"], "warn");
        assert_eq!(json[2]["level"], "fail");
    }
}
