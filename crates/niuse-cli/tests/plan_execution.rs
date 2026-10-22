//! Multi-turn plan execution: one step at a time, verification between steps.
//!
//! The human (test) decides when a step is verified and closed. The next turn
//! is composed from the remaining plan, so a completed step disappears from the
//! prompt instead of lingering as a stale instruction.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use niucode_agent::{run_with_history, AgentConfig, ApprovalMode, ApprovalRequest};
use niucode_core::{load, now_ms, Event, SessionLog};
use niucode_mcp::{McpHost, McpServerSpec};
use niucode_provider::{ScriptedProvider, StreamEvent, ToolCallDelta};
use niucode_tools::{NetworkMode, SandboxMode, SandboxPolicy};
use niuse_core::{compose_with_plan, Date, Store};
use serde_json::json;

fn pack(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packs")
        .join(name)
        .join("server.py")
}

fn env(extra: &[(&str, String)]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    map.insert(
        "PATH".to_string(),
        std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string()),
    );
    for (key, value) in extra {
        map.insert(key.to_string(), value.clone());
    }
    map
}

fn tool_turn(id: &str, name: &str, arguments: serde_json::Value) -> Vec<StreamEvent> {
    vec![
        StreamEvent::ToolCallDelta(ToolCallDelta {
            index: 0,
            id: Some(id.into()),
            name: Some(name.into()),
            arguments: arguments.to_string(),
        }),
        StreamEvent::Done {
            finish_reason: Some("tool_calls".into()),
        },
    ]
}

fn text_turn(text: &str) -> Vec<StreamEvent> {
    vec![
        StreamEvent::TextDelta(text.into()),
        StreamEvent::Done {
            finish_reason: Some("stop".into()),
        },
    ]
}

async fn run_turn(
    log: &mut SessionLog,
    host: &Arc<McpHost>,
    system: String,
    turns: Vec<Vec<StreamEvent>>,
    prompt: &str,
) -> String {
    let provider = ScriptedProvider::from_turns(turns);
    let mut config = AgentConfig::new("scripted", log.path().parent().unwrap());
    config.provider = "scripted".into();
    config.system_prompt = system;
    config.approval = ApprovalMode::Never;
    config.sandbox = SandboxPolicy::new(SandboxMode::ReadOnly, NetworkMode::Off);
    config.mcp_host = Some(host.clone());
    run_with_history(
        &provider,
        log,
        &config,
        prompt,
        Vec::new(),
        &mut |_request: &ApprovalRequest| false,
        &mut |_event| {},
    )
    .await
    .expect("turn")
    .final_text
}

fn system_prompt(store: &Store) -> String {
    compose_with_plan(
        None,
        None,
        &[],
        &store.plan().unwrap(),
        &[],
        &[],
        &[],
        Date::from_days(20_000),
    )
    .system
}

#[tokio::test]
async fn steps_execute_one_at_a_time_and_vanish_after_verification() {
    let directory = tempfile::tempdir().unwrap();
    let notes = directory.path().join("notes");
    std::fs::create_dir_all(&notes).unwrap();
    std::fs::write(notes.join("standup.md"), "Rust standup note\n").unwrap();
    let calendar =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs/calendar/tests/sample.ics");

    let host = Arc::new(
        McpHost::mount(
            &[
                McpServerSpec {
                    name: "notes".into(),
                    command: "python3".into(),
                    args: vec![pack("notes").display().to_string()],
                    env: env(&[("NOTES_DIR", notes.display().to_string())]),
                    read_only: true,
                },
                McpServerSpec {
                    name: "calendar".into(),
                    command: "python3".into(),
                    args: vec![pack("calendar").display().to_string()],
                    env: env(&[
                        ("CALENDAR_ICS", calendar.display().to_string()),
                        ("CALENDAR_TODAY", "2026-10-22".to_string()),
                    ]),
                    read_only: true,
                },
            ],
            Duration::from_secs(5),
        )
        .await
        .expect("mount"),
    );

    let store = Store::at(directory.path().join(".niuse"));
    store.init().unwrap();
    let first = store
        .add_step("find the standup note".into(), Date::from_days(20_000))
        .unwrap();
    let second = store
        .add_step("check the holiday".into(), Date::from_days(20_000))
        .unwrap();

    let prompt = system_prompt(&store);
    assert!(prompt.contains("p-1 find the standup note"));
    assert!(prompt.contains("p-2 check the holiday"));

    let mut log = SessionLog::create(directory.path()).unwrap();
    let first_answer = run_turn(
        &mut log,
        &host,
        prompt,
        vec![
            tool_turn(
                "notes-1",
                "mcp__notes__search_notes",
                json!({"query": "standup"}),
            ),
            text_turn("step one done"),
        ],
        "do step one",
    )
    .await;
    assert_eq!(first_answer, "step one done");

    log.append(&Event::Verification {
        command: "grep -q standup notes/standup.md".into(),
        ok: true,
        output: "ok".into(),
        step: Some(first.label()),
        ts: now_ms(),
    })
    .unwrap();
    store.close_step(first.id).unwrap();

    let prompt = system_prompt(&store);
    assert!(!prompt.contains("p-1"));
    assert!(prompt.contains("p-2 check the holiday"));

    let second_answer = run_turn(
        &mut log,
        &host,
        prompt,
        vec![
            tool_turn(
                "calendar-1",
                "mcp__calendar__search_events",
                json!({"query": "holiday"}),
            ),
            text_turn("step two done"),
        ],
        "do step two",
    )
    .await;
    assert_eq!(second_answer, "step two done");

    log.append(&Event::Verification {
        command: "grep -q Holiday calendar.ics".into(),
        ok: true,
        output: "ok".into(),
        step: Some(second.label()),
        ts: now_ms(),
    })
    .unwrap();
    store.close_step(second.id).unwrap();

    let prompt = system_prompt(&store);
    assert!(
        !prompt.contains("当前计划"),
        "all steps done but plan remains"
    );

    let events = load(log.path()).unwrap().events;
    let results: Vec<&str> = events
        .iter()
        .filter_map(|event| match event {
            Event::ToolResult { output, ok, .. } if *ok => Some(output.as_str()),
            _ => None,
        })
        .collect();
    assert!(results.iter().any(|output| output.contains("standup.md")));
    assert!(results.iter().any(|output| output.contains("Holiday")));

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
    assert_eq!(report["plan"].as_array().unwrap().len(), 2);
    assert!(report["plan"]
        .as_array()
        .unwrap()
        .iter()
        .all(|step| step["done"] == true));
    assert_eq!(report["verifications"].as_array().unwrap().len(), 2);
    assert_eq!(report["verifications"][0]["step"], "p-1");
    assert_eq!(report["verifications"][1]["step"], "p-2");

    host.shutdown().await.unwrap();
}
