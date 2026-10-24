//! Contract tests for the niucode surface niuse depends on.
//!
//! These run offline with `ScriptedProvider`: no API key, no network. They are
//! the canary for niucode changes and the reason `niucode.rev` exists.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use niucode_agent::{
    project_history, run_with_history, AgentConfig, AgentEvent, ApprovalMode, ApprovalRequest,
};
use niucode_core::{load, now_ms, Event, SessionLog};
use niucode_provider::{ScriptedProvider, StreamEvent, ToolCallDelta};
use niucode_tools::{NetworkMode, SandboxMode, SandboxPolicy};

fn config(dir: &Path) -> AgentConfig {
    let mut config = AgentConfig::new("scripted", dir);
    config.provider = "scripted".into();
    config.system_prompt = "niuse contract test".into();
    config.sandbox = SandboxPolicy::new(SandboxMode::ReadOnly, NetworkMode::Off);
    config.approval = ApprovalMode::Never;
    config
}

fn tool_turn(call_id: &str, path: &str) -> Vec<StreamEvent> {
    vec![
        StreamEvent::ToolCallDelta(ToolCallDelta {
            index: 0,
            id: Some(call_id.into()),
            name: Some("read".into()),
            arguments: format!(r#"{{"path":"{path}"}}"#),
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

#[test]
fn delegated_work_is_available_to_every_niuse_turn() {
    let names: Vec<String> = niucode_tools::Toolbox::specs()
        .into_iter()
        .map(|spec| spec.function.name)
        .collect();
    assert!(names.contains(&"subagent".to_string()), "{names:?}");
}

/// One session, two turns, two different working directories.
///
/// This pins the behaviour niuse's whole thread model rests on: `working_dir`
/// is read per run, history is projected from the shared event log, and tools
/// resolve paths against the current directory.
#[tokio::test]
async fn one_session_switches_working_directory_per_turn() {
    let dir_one = tempfile::tempdir().expect("temp dir one");
    let dir_two = tempfile::tempdir().expect("temp dir two");
    fs::write(dir_one.path().join("a.txt"), "alpha").expect("write a");
    fs::write(dir_two.path().join("b.txt"), "beta").expect("write b");

    let provider = ScriptedProvider::from_turns(vec![
        tool_turn("call-a", "a.txt"),
        text_turn("first done"),
        tool_turn("call-b", "b.txt"),
        text_turn("second done"),
    ]);

    let mut log = SessionLog::create(dir_one.path()).expect("create session");
    log.append(&Event::SessionStart {
        id: log.id().to_string(),
        cwd: dir_one.path().display().to_string(),
        model: "scripted".into(),
        provider: "scripted".into(),
        ts: now_ms(),
    })
    .expect("append start");

    fn deny(_request: &ApprovalRequest) -> bool {
        false
    }

    fn ignore(_event: AgentEvent) {}

    let first = run_with_history(
        &provider,
        &mut log,
        &config(dir_one.path()),
        "read a",
        Vec::new(),
        &mut deny,
        &mut ignore,
    )
    .await
    .expect("first turn");
    assert_eq!(first.final_text, "first done");

    // The base system prompt must reach the provider unchanged; memory
    // injection is niuse's job, not niucode's.
    let request = provider.requests().remove(0);
    assert_eq!(request.messages[0].text(), Some("niuse contract test"));

    let history =
        project_history(&load(log.path()).expect("load").events).expect("project history");
    let second = run_with_history(
        &provider,
        &mut log,
        &config(dir_two.path()),
        "read b",
        history,
        &mut deny,
        &mut ignore,
    )
    .await
    .expect("second turn");
    assert_eq!(second.final_text, "second done");

    let events = load(log.path()).expect("load final").events;
    let outputs: Vec<&str> = events
        .iter()
        .filter_map(|event| match event {
            Event::ToolResult { output, .. } => Some(output.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        outputs.iter().any(|output| output.contains("alpha")),
        "first cwd should read alpha: {outputs:?}"
    );
    assert!(
        outputs.iter().any(|output| output.contains("beta")),
        "second cwd should read beta: {outputs:?}"
    );
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::UserMessage { text, .. } if text == "read b")));
}

static SAW_PREVIEW: AtomicBool = AtomicBool::new(false);

fn deny_and_record(request: &ApprovalRequest) -> bool {
    SAW_PREVIEW.store(request.preview.is_some(), Ordering::SeqCst);
    false
}

fn ignore_event(_event: AgentEvent) {}

/// Approval is a security boundary niuse renders but does not implement.
/// A denial must stop the mutation and still carry a structured preview.
#[tokio::test]
async fn denying_approval_blocks_the_write_and_exposes_a_preview() {
    let directory = tempfile::tempdir().expect("temp dir");
    let target = directory.path().join("created.txt");

    let provider = ScriptedProvider::from_turns(vec![
        vec![
            StreamEvent::ToolCallDelta(ToolCallDelta {
                index: 0,
                id: Some("call-write".into()),
                name: Some("write".into()),
                arguments: r#"{"path":"created.txt","content":"hello"}"#.into(),
            }),
            StreamEvent::Done {
                finish_reason: Some("tool_calls".into()),
            },
        ],
        text_turn("denied"),
    ]);

    let mut log = SessionLog::create(directory.path()).expect("create session");
    log.append(&Event::SessionStart {
        id: log.id().to_string(),
        cwd: directory.path().display().to_string(),
        model: "scripted".into(),
        provider: "scripted".into(),
        ts: now_ms(),
    })
    .expect("append start");

    let mut config = config(directory.path());
    config.approval = ApprovalMode::Ask;
    config.sandbox = SandboxPolicy::new(SandboxMode::WorkspaceWrite, NetworkMode::Off);

    let outcome = run_with_history(
        &provider,
        &mut log,
        &config,
        "write a file",
        Vec::new(),
        &mut deny_and_record,
        &mut ignore_event,
    )
    .await
    .expect("turn completes after denial");

    assert_eq!(outcome.final_text, "denied");
    assert!(!target.exists(), "a denied write must not touch disk");
    assert!(
        SAW_PREVIEW.load(Ordering::SeqCst),
        "the approver should receive a structured preview"
    );
}

#[tokio::test]
async fn compaction_handoff_preserves_verification_steps_and_edit_diagnostics_work() {
    let events = vec![
        Event::UserMessage {
            text: "implement step 1".into(),
            attachments: Vec::new(),
            ts: 1,
        },
        Event::AssistantMessage {
            text: "verifying step 1".into(),
            reasoning: None,
            tool_calls: Vec::new(),
            ts: 2,
        },
        Event::Verification {
            command: "cargo test".into(),
            ok: true,
            output: "all green".into(),
            step: Some("p-1".into()),
            ts: 3,
        },
        Event::UserMessage {
            text: "now step 2".into(),
            attachments: Vec::new(),
            ts: 4,
        },
        Event::AssistantMessage {
            text: "starting step 2".into(),
            reasoning: None,
            tool_calls: Vec::new(),
            ts: 5,
        },
    ];
    let handoff = niucode_agent::build_handoff(&events, 1).expect("build handoff");
    assert!(
        handoff.summary.contains("- pass [p-1] `cargo test`"),
        "handoff should preserve step verification: {}",
        handoff.summary
    );

    let directory = tempfile::tempdir().expect("temp dir");
    fs::write(
        directory.path().join("lib.rs"),
        "fn run() {\n    let x = 1;\n}\n",
    )
    .expect("write fixture");
    let toolbox = niucode_tools::Toolbox::with_policy(
        directory.path(),
        SandboxPolicy::new(SandboxMode::FullAccess, NetworkMode::On),
    )
    .expect("toolbox");
    let _ = toolbox
        .execute("read", &serde_json::json!({"path": "lib.rs"}))
        .await;
    let failed_edit = toolbox
        .execute(
            "edit",
            &serde_json::json!({
                "path": "lib.rs",
                "old": "fn run() {\n  let x = 1;",
                "new": "fn run() {\n    let x = 2;"
            }),
        )
        .await;
    assert!(!failed_edit.ok);
    assert!(
        failed_edit.output.contains("hint: lines 1-2 match"),
        "edit should diagnose indentation mismatch: {}",
        failed_edit.output
    );
}
