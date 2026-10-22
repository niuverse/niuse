//! `niuse verify`: run one command, print it, and append durable evidence.

use std::io::Write;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use niucode_core::{now_ms, Event, SessionLog};

const MAX_OUTPUT: usize = 4_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifyOutcome {
    pub ok: bool,
    pub code: i32,
}

pub fn verify_with_step(
    thread: &Path,
    command: &[String],
    step: Option<&str>,
    json: bool,
) -> Result<VerifyOutcome> {
    if command.is_empty() {
        bail!("verify needs a command, for example: niuse verify -- cargo test");
    }
    verify_shell_with_step(thread, &command.join(" "), step, json)
}

pub fn verify_shell(thread: &Path, shell: &str, json: bool) -> Result<VerifyOutcome> {
    verify_shell_with_step(thread, shell, None, json)
}

pub fn verify_shell_with_step(
    thread: &Path,
    shell: &str,
    step: Option<&str>,
    json: bool,
) -> Result<VerifyOutcome> {
    let step = step.map(str::trim).filter(|step| !step.is_empty());
    let shell = shell.to_string();
    if shell.trim().is_empty() {
        bail!("verify needs a command, for example: niuse verify -- cargo test");
    }
    let output = Command::new("sh")
        .arg("-c")
        .arg(&shell)
        .output()
        .with_context(|| format!("run verification command: {shell}"))?;
    let ok = output.status.success();
    let code = output.status.code().unwrap_or(-1);
    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        combined.push_str(&stderr);
    }
    let tail = tail(&combined, MAX_OUTPUT);

    let log = SessionLog::open(thread)
        .with_context(|| format!("open thread for verification {}", thread.display()))?;
    log.append(&Event::Verification {
        command: shell.clone(),
        ok,
        output: tail.clone(),
        step: step.map(ToString::to_string),
        ts: now_ms(),
    })?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "command": shell,
                "ok": ok,
                "code": code,
                "output": tail,
                "step": step,
                "thread": thread.display().to_string(),
            }))?
        );
    } else {
        print!("{tail}");
        if !tail.is_empty() && !tail.ends_with('\n') {
            println!();
        }
        println!(
            "verify · {} · {shell}",
            if ok { "passed" } else { "FAILED" }
        );
    }
    let _ = std::io::stdout().flush();
    Ok(VerifyOutcome {
        ok,
        code: if ok { 0 } else { code.max(1) },
    })
}

fn tail(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut chars: Vec<char> = text.chars().rev().take(limit).collect();
    chars.reverse();
    format!("…{}", chars.into_iter().collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::tail;

    #[test]
    fn tail_keeps_the_end_and_boundary_safe() {
        assert_eq!(tail("short", 10), "short");
        let text = "第一行\n第二行\n第三行";
        let result = tail(text, 4);
        assert!(result.starts_with('…'));
        assert_eq!(result.chars().count(), 5);
        assert!(result.ends_with("第三行"));
    }
}
