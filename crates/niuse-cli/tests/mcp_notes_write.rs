//! End-to-end mutating notes pack: denial writes nothing, approval writes once.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use niucode_agent::{run_with_history, AgentConfig, ApprovalMode, ApprovalRequest};
use niucode_core::{load, Event, SessionLog};
use niucode_mcp::{McpHost, McpServerSpec};
use niucode_provider::{ScriptedProvider, StreamEvent, ToolCallDelta};
use serde_json::json;

fn notes_pack() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs/notes/server.py")
}

fn spec(notes: &Path) -> McpServerSpec {
    let mut env = BTreeMap::new();
    env.insert("NOTES_DIR".to_string(), notes.display().to_string());
    env.insert("NOTES_WRITE".to_string(), "1".to_string());
    env.insert(
        "PATH".to_string(),
        std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string()),
    );
    McpServerSpec {
        name: "notes".to_string(),
        command: "python3".to_string(),
        args: vec![notes_pack().display().to_string()],
        env,
        read_only: false,
        write_tools: vec!["create_note".to_string()],
    }
}

fn provider() -> ScriptedProvider {
    ScriptedProvider::from_turns(vec![
        vec![
            StreamEvent::ToolCallDelta(ToolCallDelta {
                index: 0,
                id: Some("call-note".into()),
                name: Some("mcp__notes__create_note".into()),
                arguments: json!({"title": "write test", "text": "approved body"}).to_string(),
            }),
            StreamEvent::Done {
                finish_reason: Some("tool_calls".into()),
            },
        ],
        vec![
            StreamEvent::TextDelta("done".into()),
            StreamEvent::Done {
                finish_reason: Some("stop".into()),
            },
        ],
    ])
}

async fn run(
    host: Arc<McpHost>,
    directory: &Path,
    allow: bool,
) -> (Option<ApprovalRequest>, Vec<Event>) {
    let mut log = SessionLog::create(directory).unwrap();
    let mut config = AgentConfig::new("scripted", directory);
    config.provider = "scripted".into();
    config.approval = ApprovalMode::Ask;
    config.mcp_host = Some(host);
    let mut seen = None;
    run_with_history(
        &provider(),
        &mut log,
        &config,
        "write a note",
        Vec::new(),
        &mut |request: &ApprovalRequest| {
            seen = Some(request.clone());
            allow
        },
        &mut |_event| {},
    )
    .await
    .unwrap();
    let loaded = load(log.path()).unwrap();
    (seen, loaded.events)
}

#[tokio::test]
async fn denial_writes_nothing_and_approval_writes_once() {
    let notes = tempfile::tempdir().unwrap();
    let limit = Duration::from_secs(5);
    let host = Arc::new(
        McpHost::mount_with(&[spec(notes.path())], limit, true)
            .await
            .expect("write mount"),
    );

    let denied_dir = tempfile::tempdir().unwrap();
    let (request, events) = run(host.clone(), denied_dir.path(), false).await;
    let request = request.expect("approval requested");
    assert_eq!(request.tool, "mcp__notes__create_note");
    assert!(
        request.summary.contains("mutating MCP"),
        "{}",
        request.summary
    );
    assert!(
        request.summary.contains("approved body"),
        "{}",
        request.summary
    );
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Approval { tool, allowed: false, .. } if tool == "mcp__notes__create_note"
    )));
    assert!(!notes.path().join("write-test.md").exists());
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::ToolResult { output, .. } if output.contains("created write-test.md")
    )));

    let allowed_dir = tempfile::tempdir().unwrap();
    let (_, events) = run(host.clone(), allowed_dir.path(), true).await;
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Approval { tool, allowed: true, .. } if tool == "mcp__notes__create_note"
    )));
    let note = std::fs::read_to_string(notes.path().join("write-test.md")).unwrap();
    assert!(note.contains("# write test"), "{note}");
    assert!(note.contains("approved body"), "{note}");
    assert!(events.iter().any(|event| matches!(
        event,
        Event::ToolResult { output, .. } if output.contains("created write-test.md")
    )));
    host.shutdown().await.unwrap();
}
