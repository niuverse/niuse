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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Delegation {
    pub id: String,
    pub profile: String,
    pub task: String,
    pub session: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok: Option<bool>,
    pub summary: String,
    pub turns: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct McpMount {
    pub server: String,
    pub config_hash: String,
    pub write_tools: Vec<String>,
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
    pub delegations: Vec<Delegation>,
    pub mounts: Vec<McpMount>,
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
            Event::SubagentSpawn {
                id,
                task,
                profile,
                session,
                ..
            } => {
                evidence.delegations.push(Delegation {
                    id: id.clone(),
                    profile: profile.clone(),
                    task: task.clone(),
                    session: session.clone(),
                    ok: None,
                    summary: String::new(),
                    turns: 0,
                });
            }
            Event::SubagentResult {
                id,
                ok,
                summary,
                session,
                turns,
                ..
            } => {
                if let Some(run) = evidence.delegations.iter_mut().find(|run| &run.id == id) {
                    run.ok = Some(*ok);
                    run.summary = summary.clone();
                    run.session = session.clone();
                    run.turns = *turns;
                } else {
                    evidence.delegations.push(Delegation {
                        id: id.clone(),
                        profile: String::new(),
                        task: String::new(),
                        session: session.clone(),
                        ok: Some(*ok),
                        summary: summary.clone(),
                        turns: *turns,
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
            Event::McpMount {
                server,
                config_hash,
                write_tools,
                ..
            } => {
                evidence.mounts.push(McpMount {
                    server: server.clone(),
                    config_hash: config_hash.clone(),
                    write_tools: write_tools.clone(),
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
            && self.delegations.is_empty()
            && self.mounts.is_empty()
            && self.denied == 0
    }

    /// `verified` if any recorded check passed, `verification failed` if only
    /// failures exist, and `no recorded verification` when the step has none.
    pub fn step_state(&self, step: &PlanStep) -> &'static str {
        let label = step.label();
        let checks: Vec<&Check> = self
            .verifications
            .iter()
            .filter(|check| check.step.as_deref() == Some(label.as_str()))
            .collect();
        if checks.iter().any(|check| check.ok) {
            "verified"
        } else if checks.is_empty() {
            "no recorded verification"
        } else {
            "verification failed"
        }
    }

    pub fn render(&self) -> String {
        if self.is_empty() {
            return "evidence · no tool calls in this thread\n".to_string();
        }
        let passed = self.checks.iter().filter(|check| check.ok).count();
        let mut out = format!(
            "evidence · {} tool call(s) · {} file(s) · {}/{} check(s) · {} verification(s) · {} delegated · {} mcp mount(s) · {} failure(s)\n",
            self.tools,
            self.files.len(),
            passed,
            self.checks.len(),
            self.verifications.len(),
            self.delegations.len(),
            self.mounts.len(),
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
        if !self.delegations.is_empty() {
            out.push_str("delegated\n");
            for run in &self.delegations {
                let status = match run.ok {
                    Some(true) => "✓",
                    Some(false) => "✗",
                    None => "?",
                };
                out.push_str(&format!(
                    "  {status} {} · {} · {} turn(s) · {}\n",
                    if run.profile.is_empty() {
                        "subagent"
                    } else {
                        &run.profile
                    },
                    run.task,
                    run.turns,
                    run.session
                ));
                if !run.summary.is_empty() {
                    out.push_str(&format!("    {}\n", run.summary));
                }
            }
        }
        if !self.plan.is_empty() {
            out.push_str("plan\n");
            for step in &self.plan {
                out.push_str(&format!(
                    "  [{}] {} {} — {}\n",
                    if step.done { "x" } else { " " },
                    step.label(),
                    step.text,
                    self.step_state(step)
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
        if !self.mounts.is_empty() {
            out.push_str("mcp\n");
            for mount in &self.mounts {
                let writes = if mount.write_tools.is_empty() {
                    "read-only".to_string()
                } else {
                    format!("writes: {}", mount.write_tools.join(", "))
                };
                out.push_str(&format!(
                    "  {} · {} · {}\n",
                    mount.server, writes, mount.config_hash
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
            event(
                r#"{"type":"subagent_spawn","id":"s1","call_id":"c1","task":"inspect auth","profile":"explore","session":"sub.jsonl","max_steps":6,"timeout_secs":120,"context_budget":16000,"ts":7}"#,
            ),
            event(
                r#"{"type":"subagent_result","id":"s1","call_id":"c1","ok":true,"summary":"found the seam","session":"sub.jsonl","turns":2,"usage":{},"ts":8}"#,
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
        assert_eq!(evidence.delegations.len(), 1);
        assert_eq!(evidence.delegations[0].ok, Some(true));
        assert!(rendered.contains("inspect auth"));
        assert!(rendered.contains("found the seam"));
    }

    #[test]
    fn an_unfinished_delegation_is_not_reported_as_success() {
        let events = vec![event(
            r#"{"type":"subagent_spawn","id":"s2","call_id":"c2","task":"slow task","profile":"explore","session":"sub.jsonl","max_steps":6,"timeout_secs":120,"context_budget":16000,"ts":1}"#,
        )];
        let evidence = from_events(&events);
        assert_eq!(evidence.delegations[0].ok, None);
        let rendered = evidence.render();
        assert!(rendered.contains("? explore"));
        assert!(rendered.contains("slow task"));
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
        assert_eq!(evidence.step_state(&evidence.plan[0]), "verified");
        assert_eq!(
            evidence.step_state(&evidence.plan[1]),
            "no recorded verification"
        );
    }

    #[test]
    fn a_failed_step_check_is_not_reported_as_missing() {
        let step = PlanStep::new(1, "fix the check", niuse_core::Date::from_days(20_000));
        let evidence = Evidence {
            plan: vec![step.clone()],
            verifications: vec![Check {
                command: "cargo test".into(),
                ok: false,
                step: Some("p-1".into()),
            }],
            ..Evidence::default()
        };
        assert_eq!(evidence.step_state(&step), "verification failed");
        assert!(evidence.render().contains("verification failed"));
    }

    #[test]
    fn mcp_mounts_show_their_write_allowlist() {
        let evidence = from_events(&[event(
            r#"{"type":"mcp_mount","server":"notes","config_hash":"0123456789abcdef","write_tools":["mcp__notes__create_note"],"ts":1}"#,
        )]);
        let rendered = evidence.render();
        assert!(rendered.contains("1 mcp mount(s)"));
        assert!(rendered.contains("notes · writes: mcp__notes__create_note"));
        assert!(rendered.contains("0123456789abcdef"));
        assert!(!evidence.is_empty());
    }

    #[test]
    fn an_empty_thread_says_so() {
        assert!(from_events(&[]).is_empty());
        assert!(from_events(&[]).render().contains("no tool calls"));
    }
}
