//! A read-only `memory_search` tool for the agent.
//!
//! It reuses the human-facing unified recall path, so the model and the user
//! see the same ranking, freshness labels, and caps. The tool cannot write,
//! delete, or confirm memory.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use niucode_agent::EmbedderTool;
use niucode_provider::ToolSpec;
use niucode_tools::ToolResult;
use niuse_core::Store;
use serde_json::{json, Value};

#[derive(Debug)]
pub struct MemoryTool {
    root: PathBuf,
}

impl MemoryTool {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

pub fn tools(root: PathBuf) -> Vec<Arc<dyn EmbedderTool>> {
    vec![Arc::new(MemoryTool::new(root))]
}

#[async_trait]
impl EmbedderTool for MemoryTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec::function(
            "memory_search",
            "Search the user's durable memory, journal, promises, and goals, including entries no longer injected. Read-only. Use when an old judgement may be missing; cite returned ids. Results are capped.",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Short keywords, not a full question"},
                    "limit": {"type": "integer", "description": "Maximum hits", "minimum": 1, "maximum": 50}
                },
                "required": ["query"]
            }),
        )
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let Some(query) = input
            .get("query")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|query| !query.is_empty())
        else {
            return ToolResult {
                ok: false,
                output: "memory_search requires a non-empty query".into(),
            };
        };
        let limit = input
            .get("limit")
            .and_then(Value::as_u64)
            .unwrap_or(10)
            .clamp(1, 50) as usize;
        let store = Store::at(self.root.clone());
        match crate::commands::recall_text(&store, query, limit) {
            Ok(text) => ToolResult {
                ok: true,
                output: text,
            },
            Err(error) => ToolResult {
                ok: false,
                output: format!("memory search failed: {error:#}"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use niucode_agent::{run_with_history, AgentConfig, ApprovalMode, ApprovalRequest};
    use niucode_core::{load, Event, SessionLog};
    use niucode_provider::{ScriptedProvider, StreamEvent, ToolCallDelta};
    use niuse_core::{Date, MemoryEntry, MemoryKind};

    fn store() -> (tempfile::TempDir, PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join(".niuse");
        let store = Store::at(root.clone());
        store.init().unwrap();
        store
            .add_memory(MemoryEntry::new(
                MemoryKind::Lesson,
                "Rust projects use cargo nextest",
                vec!["rust".into()],
                Date::today(),
                None,
            ))
            .unwrap();
        (directory, root)
    }

    #[test]
    fn spec_is_named_and_read_only() {
        let tool = MemoryTool::new(PathBuf::from("/tmp/unused"));
        let spec = tool.spec();
        assert_eq!(spec.function.name, "memory_search");
        assert!(tool.read_only());
    }

    #[tokio::test]
    async fn execute_reuses_unified_recall() {
        let (_directory, root) = store();
        let tool = MemoryTool::new(root);
        let result = tool.execute(json!({"query": "rust"})).await;
        assert!(result.ok, "{}", result.output);
        assert!(result.output.contains("cargo nextest"));
        assert!(result.output.contains("m-"));
    }

    #[tokio::test]
    async fn missing_query_is_an_error() {
        let (_directory, root) = store();
        let tool = MemoryTool::new(root);
        let result = tool.execute(json!({})).await;
        assert!(!result.ok);
        assert!(result.output.contains("non-empty query"));
    }

    #[tokio::test]
    async fn a_turn_can_call_memory_search() {
        let (directory, root) = store();
        let provider = ScriptedProvider::from_turns(vec![
            vec![
                StreamEvent::ToolCallDelta(ToolCallDelta {
                    index: 0,
                    id: Some("call-memory".into()),
                    name: Some("memory_search".into()),
                    arguments: json!({"query": "rust"}).to_string(),
                }),
                StreamEvent::Done {
                    finish_reason: Some("tool_calls".into()),
                },
            ],
            vec![
                StreamEvent::TextDelta("cited the result".into()),
                StreamEvent::Done {
                    finish_reason: Some("stop".into()),
                },
            ],
        ]);
        let mut log = SessionLog::create(directory.path()).unwrap();
        let mut config = AgentConfig::new("scripted", directory.path());
        config.provider = "scripted".into();
        config.approval = ApprovalMode::Never;
        config.extra_tools = tools(root);

        let outcome = run_with_history(
            &provider,
            &mut log,
            &config,
            "find the old rust decision",
            Vec::new(),
            &mut |_request: &ApprovalRequest| false,
            &mut |_event| {},
        )
        .await
        .unwrap();

        assert_eq!(outcome.final_text, "cited the result");
        let loaded = load(log.path()).unwrap();
        let tool_output = loaded
            .events
            .iter()
            .find_map(|event| match event {
                Event::ToolResult { output, .. } => Some(output.as_str()),
                _ => None,
            })
            .expect("tool result event");
        assert!(tool_output.contains("cargo nextest"), "{tool_output}");
        assert!(tool_output.contains("m-"), "{tool_output}");
    }
}
