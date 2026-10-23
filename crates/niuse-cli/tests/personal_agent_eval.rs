//! Offline personal-agent evaluation suite.
//!
//! These are prompt-and-turn regressions, not a model-quality benchmark. Each
//! case names one personal capability so a failure says what regressed.

use std::path::Path;

use async_trait::async_trait;
use futures_util::stream;
use niucode_agent::{run_with_history, AgentConfig, ApprovalMode, ApprovalRequest};
use niucode_core::SessionLog;
use niucode_provider::{
    ChatMessage, CompletionRequest, EventStream, Provider, ProviderError, StreamEvent,
};
use niucode_tools::{NetworkMode, SandboxMode, SandboxPolicy};
use niuse_core::{
    compose_with_goals, compose_with_plan, Brief, Date, Goal, LoopItem, MemoryEntry, MemoryKind,
    PlanStep, Store,
};

/// Answers differently depending on whether the injected prompt contains a
/// remembered judgement. This makes the A/B a behavioural test, not a string
/// comparison.
#[derive(Clone)]
struct EvalProvider;

#[async_trait]
impl Provider for EvalProvider {
    fn name(&self) -> &str {
        "eval"
    }

    async fn stream(&self, request: CompletionRequest) -> Result<EventStream, ProviderError> {
        let system = request
            .messages
            .first()
            .and_then(|message: &ChatMessage| message.text().map(str::to_string))
            .unwrap_or_default();
        let text = if system.contains("cargo nextest") {
            "I remember: this repo uses cargo nextest."
        } else {
            "I do not have a memory for that."
        };
        Ok(Box::pin(stream::iter(vec![
            Ok(StreamEvent::TextDelta(text.into())),
            Ok(StreamEvent::Done {
                finish_reason: Some("stop".into()),
            }),
        ])))
    }
}

fn config(system: String, directory: &Path) -> AgentConfig {
    let mut config = AgentConfig::new("eval", directory);
    config.provider = "eval".into();
    config.system_prompt = system;
    config.approval = ApprovalMode::Never;
    config.sandbox = SandboxPolicy::new(SandboxMode::ReadOnly, NetworkMode::Off);
    config
}

async fn answer(system: String, directory: &Path) -> String {
    let provider = EvalProvider;
    let mut log = SessionLog::create(directory).expect("session");
    let outcome = run_with_history(
        &provider,
        &mut log,
        &config(system, directory),
        "what do you remember?",
        Vec::new(),
        &mut |_request: &ApprovalRequest| false,
        &mut |_event| {},
    )
    .await
    .expect("turn");
    outcome.final_text
}

#[tokio::test]
async fn memory_on_off_changes_the_answer_and_the_cited_text() {
    let directory = tempfile::tempdir().unwrap();
    let today = Date::today();
    let memory = MemoryEntry::new(
        MemoryKind::Lesson,
        "Rust projects use cargo nextest",
        vec!["rust".into()],
        today,
        None,
    );

    let with_memory = compose_with_goals(None, None, &[], &[memory], &[], &[], today);
    let without_memory = compose_with_goals(None, None, &[], &[], &[], &[], today);
    assert!(with_memory.system.contains("cargo nextest"));
    assert!(!without_memory.system.contains("cargo nextest"));

    let on = answer(with_memory.system, directory.path()).await;
    let off = answer(without_memory.system, directory.path()).await;
    assert!(on.contains("cargo nextest"), "{on}");
    assert!(!off.contains("cargo nextest"), "{off}");
    assert_ne!(on, off, "memory must change behaviour");
}

#[test]
fn goal_injection_survives_only_while_open() {
    let today = Date::today();
    let open = Goal::new("ship the personal-agent eval", today);
    let closed = Goal {
        done: true,
        ..open.clone()
    };
    let on = compose_with_goals(None, None, &[open], &[], &[], &[], today);
    assert!(on.system.contains("当前目标"));
    assert!(on.system.contains("ship the personal-agent eval"));
    assert!(on.system.contains("g-"));

    let off = compose_with_goals(None, None, &[closed], &[], &[], &[], today);
    assert!(!off.system.contains("当前目标"));
}

#[test]
fn plan_prompt_injects_open_steps_and_skips_done_ones() {
    let today = Date::today();
    let open = PlanStep::new(1, "write the eval", today);
    let mut done = PlanStep::new(2, "old step", today);
    done.done = true;
    let composition = compose_with_plan(None, None, &[], &[open, done], &[], &[], &[], today);
    assert!(composition.system.contains("当前计划"));
    assert!(composition.system.contains("p-1 write the eval"));
    assert!(!composition.system.contains("p-2"));
    assert!(composition.system.contains("niuse verify --step p-N"));
}

#[test]
fn brief_surfaces_overdue_promises_and_stale_memory() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::at(directory.path().join(".niuse"));
    store.init().unwrap();
    let today = Date::today();

    let mut stale = MemoryEntry::new(
        MemoryKind::Preference,
        "old preference",
        vec![],
        today,
        None,
    );
    stale.confirmed = today.plus_days(-40);
    store.add_memory(stale).unwrap();
    store
        .add_loop(LoopItem::new("ship the eval", Some(today.plus_days(-1))))
        .unwrap();
    store.write_inbox("- [ ] pending\n").unwrap();

    let brief = Brief::build(&store, today, "niuse").unwrap();
    assert_eq!(brief.open_loops, 1);
    assert_eq!(brief.open_goals, 0);
    assert_eq!(brief.stale, 1);
    assert_eq!(brief.inbox, 1);
    let lines = brief.startup_lines().join("\n");
    assert!(lines.contains("1 条逾期承诺"));
    assert!(lines.contains("1 条待审查"));
    assert!(lines.contains("ship the eval"));
}
