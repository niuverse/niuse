//! End-to-end test for the read-only notes MCP pack.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use niucode_mcp::{McpHost, McpServerSpec};
use serde_json::json;

fn server() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs/notes/server.py")
}

#[tokio::test]
async fn notes_pack_searches_reads_and_refuses_escape() {
    let notes = tempfile::tempdir().unwrap();
    std::fs::write(notes.path().join("roadmap.md"), "Rust note\nsecond line\n").unwrap();
    std::fs::write(notes.path().join("other.txt"), "nothing here\n").unwrap();

    let mut env = BTreeMap::new();
    env.insert("NOTES_DIR".to_string(), notes.path().display().to_string());
    env.insert(
        "PATH".to_string(),
        std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string()),
    );
    let spec = McpServerSpec {
        name: "notes".to_string(),
        command: "python3".to_string(),
        args: vec![server().display().to_string()],
        env,
        read_only: true,
        write_tools: Vec::new(),
    };

    let limit = Duration::from_secs(5);
    let host = McpHost::mount(&[spec], limit).await.expect("mount");
    let names: Vec<String> = host
        .specs()
        .into_iter()
        .map(|spec| spec.function.name)
        .collect();
    assert!(names.contains(&"mcp__notes__search_notes".to_string()));

    let search = host
        .call("mcp__notes__search_notes", json!({"query": "rust"}), limit)
        .await
        .expect("search");
    assert!(search.ok);
    assert!(search.output.contains("roadmap.md:1: Rust note"));

    let read = host
        .call(
            "mcp__notes__read_note",
            json!({"path": "roadmap.md"}),
            limit,
        )
        .await
        .expect("read");
    assert!(read.ok);
    assert!(read.output.contains("second line"));

    let escape = host
        .call(
            "mcp__notes__read_note",
            json!({"path": "../escape.md"}),
            limit,
        )
        .await
        .expect("escape is a tool error");
    assert!(!escape.ok);
    assert!(escape.output.contains("escapes NOTES_DIR"));

    host.shutdown().await.expect("shutdown");
}
