//! Combined personal workflow: two MCP packs, a plan step, and verification.
//!
//! This is the M9 cross-layer case: the model reads two real personal sources,
//! the human records a step-scoped verification, and the step closes only then.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use niucode_agent::{run_with_history, AgentConfig, ApprovalMode, ApprovalRequest};
use niucode_core::{load, now_ms, Event};
use niucode_mcp::{McpHost, McpServerSpec};
use niucode_provider::{ScriptedProvider, StreamEvent, ToolCallDelta};
use niucode_tools::{NetworkMode, SandboxMode, SandboxPolicy};
use niuse_core::{Date, Store};
use serde_json::json;

fn pack(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packs")
        .join(name)
        .join("server.py")
}

fn calendar() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs/calendar/tests/sample.ics")
}

fn env(extra: &[(&str, String)]) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    env.insert(
        "PATH".to_string(),
        std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string()),
    );
    for (key, value) in extra {
        env.insert(key.to_string(), value.clone());
    }
    env
}

fn spec(name: &str, args: Vec<String>, envs: BTreeMap<String, String>) -> McpServerSpec {
    McpServerSpec {
        name: name.to_string(),
        command: "python3".to_string(),
        args,
        env: envs,
        read_only: true,
    }
}

#[tokio::test]
async fn two_packs_plan_step_and_verification_close_the_loop() {
    let directory = tempfile::tempdir().unwrap();
    let notes = directory.path().join("notes");
    std::fs::create_dir_all(&notes).unwrap();
    std::fs::write(notes.join("standup.md"), "Rust standup note\n").unwrap();

    let notes_spec = spec(
        "notes",
        vec![pack("notes").display().to_string()],
        env(&[("NOTES_DIR", notes.display().to_string())]),
    );
    let calendar_spec = spec(
        "calendar",
        vec![pack("calendar").display().to_string()],
        env(&[
            ("CALENDAR_ICS", calendar().display().to_string()),
            ("CALENDAR_TODAY", "2026-10-22".to_string()),
        ]),
    );
    let limit = Duration::from_secs(5);
    let host = Arc::new(
        McpHost::mount(&[notes_spec, calendar_spec], limit)
            .await
            .expect("mount both packs"),
    );

    let store = Store::at(directory.path().join(".niuse"));
    store.init().unwrap();
    let step = store
        .add_step("find the standup note".into(), Date::from_days(20_000))
        .unwrap();

    let provider = ScriptedProvider::from_turns(vec![
        vec![
            StreamEvent::ToolCallDelta(ToolCallDelta {
                index: 0,
                id: Some("calendar-1".into()),
                name: Some("mcp__calendar__search_events".into()),
                arguments: json!({"query": "standup"}).to_string(),
            }),
            StreamEvent::Done {
                finish_reason: Some("tool_calls".into()),
            },
        ],
        vec![
            StreamEvent::ToolCallDelta(ToolCallDelta {
                index: 0,
                id: Some("notes-1".into()),
                name: Some("mcp__notes__search_notes".into()),
                arguments: json!({"query": "standup"}).to_string(),
            }),
            StreamEvent::Done {
                finish_reason: Some("tool_calls".into()),
            },
        ],
        vec![
            StreamEvent::TextDelta("found the standup in both sources".into()),
            StreamEvent::Done {
                finish_reason: Some("stop".into()),
            },
        ],
    ]);

    let mut log = niucode_core::SessionLog::create(directory.path()).unwrap();
    let mut config = AgentConfig::new("scripted", directory.path());
    config.provider = "scripted".into();
    config.approval = ApprovalMode::Never;
    config.sandbox = SandboxPolicy::new(SandboxMode::ReadOnly, NetworkMode::Off);
    config.mcp_host = Some(host.clone());

    let outcome = run_with_history(
        &provider,
        &mut log,
        &config,
        "what is on my calendar and in my notes?",
        Vec::new(),
        &mut |_request: &ApprovalRequest| false,
        &mut |_event| {},
    )
    .await
    .expect("agent turn");
    assert_eq!(outcome.final_text, "found the standup in both sources");

    let events = load(log.path()).unwrap().events;
    let outputs: Vec<&str> = events
        .iter()
        .filter_map(|event| match event {
            Event::ToolResult { output, ok, .. } if *ok => Some(output.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        outputs.iter().any(|output| output.contains("Standup")),
        "{outputs:?}"
    );
    assert!(
        outputs.iter().any(|output| output.contains("standup.md")),
        "{outputs:?}"
    );

    // The human runs the deciding check and tags it to the step.
    log.append(&Event::Verification {
        command: "grep -q standup notes/standup.md".into(),
        ok: true,
        output: "ok".into(),
        step: Some(step.label()),
        ts: now_ms(),
    })
    .unwrap();

    let closed = store.close_step(step.id).unwrap().expect("closed");
    assert!(closed.done);
    let evidence = std::process::Command::new(env!("CARGO_BIN_EXE_niuse"))
        .arg("--home")
        .arg(directory.path().join(".niuse"))
        .arg("evidence")
        .arg("--thread")
        .arg(log.path())
        .arg("--json")
        .output()
        .expect("evidence");
    assert!(evidence.status.success(), "{evidence:?}");
    let report: serde_json::Value = serde_json::from_slice(&evidence.stdout).unwrap();
    assert_eq!(report["plan"][0]["done"], true);
    assert_eq!(report["verifications"][0]["step"], "p-1");
    assert!(report["tools"].as_u64().unwrap() >= 2);

    host.shutdown().await.unwrap();
}
