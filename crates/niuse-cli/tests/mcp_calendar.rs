//! End-to-end test for the read-only calendar MCP pack.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use niucode_mcp::{McpHost, McpServerSpec};
use serde_json::json;

fn pack() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs/calendar/server.py")
}

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs/calendar/tests/sample.ics")
}

#[tokio::test]
async fn calendar_pack_lists_and_searches_without_network() {
    let mut env = BTreeMap::new();
    env.insert("CALENDAR_ICS".to_string(), fixture().display().to_string());
    env.insert("CALENDAR_TODAY".to_string(), "2026-10-22".to_string());
    env.insert(
        "PATH".to_string(),
        std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string()),
    );
    let spec = McpServerSpec {
        name: "calendar".to_string(),
        command: "python3".to_string(),
        args: vec![pack().display().to_string()],
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
    assert!(names.contains(&"mcp__calendar__list_events".to_string()));

    let list = host
        .call("mcp__calendar__list_events", json!({"days": 7}), limit)
        .await
        .expect("list");
    assert!(list.ok);
    assert!(list.output.contains("Standup, daily"));
    assert!(list.output.contains("Holiday"));
    assert!(!list.output.contains("Future event"));

    let search = host
        .call(
            "mcp__calendar__search_events",
            json!({"query": "roadmap"}),
            limit,
        )
        .await
        .expect("search");
    assert!(search.ok);
    assert!(search.output.contains("2026-10-24"));

    let miss = host
        .call(
            "mcp__calendar__search_events",
            json!({"query": "nonexistent"}),
            limit,
        )
        .await
        .expect("miss");
    assert!(miss.ok);
    assert_eq!(miss.output, "no matches");

    host.shutdown().await.expect("shutdown");
}
