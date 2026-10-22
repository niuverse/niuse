//! Execute → verify: the evidence a thread actually contains.
//!
//! This is a read-only report over tool calls and results. It does not claim
//! that a task is correct; it separates what ran and succeeded from what is
//! merely asserted in the final answer.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::Result;
use niucode_core::{load, Event};
use niuse_core::PlanStep;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Check {
    pub command: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Evidence {
    pub tools: usize,
    pub failures: usize,
    pub denied: usize,
    pub files: Vec<String>,
    pub checks: Vec<Check>,
    pub verifications: Vec<Check>,
    #[serde(default)]
    pub plan: Vec<PlanStep>,
}

pub fn from_thread(path: &Path) -> Result<Evidence> {
    let loaded = load(path)?;
    Ok(from_events(&loaded.events))
}

pub fn from_events(events: &[Event]) -> Evidence {
    let mut evidence = Evidence::default();
    let mut files = BTreeSet::new();
    let mut pending: BTreeMap<String, (String, String)> = BTreeMap::new();

    for event in events {
        match event {
            Event::AssistantMessage { tool_calls, .. } => {
                for call in tool_calls {
                    evidence.tools += 1;
                    let path = call
                        .input
                        .get("path")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let command = call
                        .input
                        .get("command")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    if matches!(call.name.as_str(), "write" | "edit") && !path.is_empty() {
                        files.insert(path);
                    }
                    pending.insert(call.id.clone(), (call.name.clone(), command));
                }
            }
            Event::ToolResult { call_id, ok, .. } => {
                let Some((name, command)) = pending.remove(call_id) else {
                    continue;
                };
                if !ok {
                    evidence.failures += 1;
                }
                if name == "bash" && looks_like_check(&command) {
                    evidence.checks.push(Check {
                        command,
                        ok: *ok,
                        step: None,
                    });
                }
            }
            Event::Verification {
                command, ok, step, ..
            } => {
                evidence.verifications.push(Check {
                    command: command.clone(),
                    ok: *ok,
                    step: step.clone(),
                });
            }
            Event::Approval { allowed, .. } if !allowed => {
                evidence.denied += 1;
            }
            _ => {}
        }
    }

    evidence.files = files.into_iter().collect();
    evidence
}

impl Evidence {
    pub fn is_empty(&self) -> bool {
        self.tools == 0
            && self.files.is_empty()
            && self.checks.is_empty()
            && self.verifications.is_empty()
            && self.plan.is_empty()
            && self.denied == 0
    }

    pub fn render(&self) -> String {
        if self.is_empty() {
            return "evidence · no tool calls in this thread\n".to_string();
        }
        let passed = self.checks.iter().filter(|check| check.ok).count();
        let mut out = format!(
            "evidence · {} tool call(s) · {} file(s) · {}/{} check(s) · {} verification(s) · {} failure(s)\n",
            self.tools,
            self.files.len(),
            passed,
            self.checks.len(),
            self.verifications.len(),
            self.failures
        );
        if !self.files.is_empty() {
            out.push_str("files\n");
            for file in &self.files {
                out.push_str(&format!("  {file}\n"));
            }
        }
        if !self.checks.is_empty() {
            out.push_str("checks\n");
            for check in &self.checks {
                out.push_str(&format!(
                    "  {} {}\n",
                    if check.ok { "✓" } else { "✗" },
                    check.command
                ));
            }
        }
        if !self.plan.is_empty() {
            out.push_str("plan\n");
            for step in &self.plan {
                let label = step.label();
                let verified = self
                    .verifications
                    .iter()
                    .any(|check| check.step.as_deref() == Some(label.as_str()) && check.ok);
                out.push_str(&format!(
                    "  [{}] {} {} — {}\n",
                    if step.done { "x" } else { " " },
                    label,
                    step.text,
                    if verified {
                        "verified"
                    } else {
                        "no recorded verification"
                    }
                ));
            }
        }
        if !self.verifications.is_empty() {
            out.push_str("verification\n");
            for check in &self.verifications {
                out.push_str(&format!(
                    "  {} {}{}\n",
                    if check.ok { "✓" } else { "✗" },
                    check
                        .step
                        .as_deref()
                        .map(|step| format!("[{step}] "))
                        .unwrap_or_default(),
                    check.command
                ));
            }
        }
        if self.denied > 0 {
            out.push_str(&format!("denied · {} approval(s)\n", self.denied));
        }
        out
    }
}

fn looks_like_check(command: &str) -> bool {
    let command = command.to_lowercase();
    [
        "test",
        "check",
        "clippy",
        "fmt",
        "lint",
        "pytest",
        "cargo build",
        "cargo xtask",
        "npm test",
        "pnpm test",
        "go test",
    ]
    .iter()
    .any(|needle| command.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(line: &str) -> Event {
        serde_json::from_str(line).unwrap()
    }

    #[test]
    fn reports_files_checks_failures_and_denials() {
        let events = vec![
            event(
                r#"{"type":"assistant_message","text":"working","tool_calls":[
                    {"id":"c1","name":"write","input":{"path":"src/lib.rs","content":"x"}},
                    {"id":"c2","name":"bash","input":{"command":"cargo test"}},
                    {"id":"c3","name":"bash","input":{"command":"cargo clippy"}}
                ],"ts":1}"#,
            ),
            event(r#"{"type":"tool_result","call_id":"c1","ok":true,"output":"ok","ts":2}"#),
            event(r#"{"type":"tool_result","call_id":"c2","ok":true,"output":"ok","ts":3}"#),
            event(r#"{"type":"tool_result","call_id":"c3","ok":false,"output":"bad","ts":4}"#),
            event(r#"{"type":"approval","tool":"bash","allowed":false,"ts":5}"#),
            event(
                r#"{"type":"verification","command":"cargo test","ok":true,"output":"ok","ts":6}"#,
            ),
        ];
        let evidence = from_events(&events);
        assert_eq!(evidence.tools, 3);
        assert_eq!(evidence.files, vec!["src/lib.rs"]);
        assert_eq!(evidence.checks.len(), 2);
        assert!(evidence.checks[0].ok);
        assert!(!evidence.checks[1].ok);
        assert_eq!(evidence.failures, 1);
        assert_eq!(evidence.denied, 1);
        assert_eq!(evidence.verifications.len(), 1);
        assert!(evidence.verifications[0].ok);
        let rendered = evidence.render();
        assert!(rendered.contains("src/lib.rs"));
        assert!(rendered.contains("cargo test"));
        assert!(rendered.contains("denied"));
        assert!(rendered.contains("verification"));
    }

    #[test]
    fn plan_steps_show_their_verification_status() {
        let evidence = Evidence {
            plan: vec![
                PlanStep::new(1, "write tests", niuse_core::Date::from_days(20_000)),
                PlanStep::new(2, "ship", niuse_core::Date::from_days(20_000)),
            ],
            verifications: vec![Check {
                command: "cargo test".into(),
                ok: true,
                step: Some("p-1".into()),
            }],
            ..Evidence::default()
        };
        let rendered = evidence.render();
        assert!(rendered.contains("p-1 write tests — verified"));
        assert!(rendered.contains("p-2 ship — no recorded verification"));
    }

    #[test]
    fn an_empty_thread_says_so() {
        assert!(from_events(&[]).is_empty());
        assert!(from_events(&[]).render().contains("no tool calls"));
    }
}
