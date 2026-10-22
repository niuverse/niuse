//! Prompt-size regressions: input tokens are the main recurring cost and the
//! largest cache-prefix risk. These cases keep a fat store bounded.

use niuse_core::prompt::{
    compose_with_plan, estimate_text_tokens, Composition, GOAL_BUDGET, JOURNAL_BUDGET, LOOP_BUDGET,
    MEMORY_BUDGET, PLAN_BUDGET,
};
use niuse_core::{Date, Goal, JournalDay, LoopItem, MemoryEntry, MemoryKind, PlanStep};

const TOTAL_BUDGET: usize = 4_200;
const SLACK: usize = 250;

fn section(system: &str, title: &str) -> String {
    let marker = format!("## {title}");
    let Some(start) = system.find(&marker) else {
        return String::new();
    };
    let body_start = start + marker.len();
    let body = &system[body_start..];
    let end = body.find("\n## ").unwrap_or(body.len());
    body[..end].to_string()
}

fn today() -> Date {
    Date::from_days(20_000)
}

fn memories(count: usize) -> Vec<MemoryEntry> {
    (0..count)
        .map(|index| {
            MemoryEntry::new(
                MemoryKind::Lesson,
                format!("lesson {index}: prefer the smaller patch and verify it"),
                vec!["budget".into()],
                today(),
                None,
            )
        })
        .collect()
}

fn goals() -> Vec<Goal> {
    (0..7)
        .map(|index| Goal::new(format!("goal {index} with a concrete finish line"), today()))
        .collect()
}

fn plan() -> Vec<PlanStep> {
    (1..=6)
        .map(|id| PlanStep::new(id, "small step with a verifiable end", today()))
        .collect()
}

fn loops() -> Vec<LoopItem> {
    (0..20)
        .map(|index| LoopItem::new(format!("promise {index}"), Some(today().plus_days(-1))))
        .collect()
}

fn journal() -> Vec<JournalDay> {
    (0..10)
        .map(|index| JournalDay {
            date: today().plus_days(-index),
            bullets: vec![
                "decided to keep the smaller patch".into(),
                "open question about the budget".into(),
            ],
        })
        .collect()
}

fn composed(count: usize) -> Composition {
    compose_with_plan(
        Some("# 你是 niubot\n短句，直接，先说结论。"),
        Some("Rust 工程师，长期做 CLI。"),
        &goals(),
        &plan(),
        &memories(count),
        &loops(),
        &journal(),
        today(),
    )
}

#[test]
fn a_fat_store_stays_under_the_prompt_budget() {
    let composition = composed(200);
    assert_eq!(
        composition.tokens,
        estimate_text_tokens(&composition.system),
        "reported tokens must match the estimator"
    );
    println!("fat prompt tokens: {}", composition.tokens);
    assert!(
        composition.tokens <= TOTAL_BUDGET,
        "prompt grew to {} tokens",
        composition.tokens
    );
    assert!(
        estimate_text_tokens(&section(&composition.system, "长期记忆")) <= MEMORY_BUDGET + SLACK
    );
    assert!(estimate_text_tokens(&section(&composition.system, "当前计划")) <= PLAN_BUDGET + SLACK);
    assert!(estimate_text_tokens(&section(&composition.system, "当前目标")) <= GOAL_BUDGET + SLACK);
    assert!(
        estimate_text_tokens(&section(&composition.system, "未完成的承诺")) <= LOOP_BUDGET + SLACK
    );
    assert!(
        estimate_text_tokens(&section(&composition.system, "最近几天")) <= JOURNAL_BUDGET + SLACK
    );
    assert!(composition.memory_dropped > 0);
}

#[test]
fn plan_injection_caps_at_six_steps() {
    let steps: Vec<PlanStep> = (1..=50)
        .map(|id| PlanStep::new(id, "bounded step", today()))
        .collect();
    let composition = compose_with_plan(None, None, &[], &steps, &[], &[], &[], today());
    let plan = section(&composition.system, "当前计划");
    assert!(plan.contains("p-1 bounded step"));
    assert!(plan.contains("p-6 bounded step"));
    assert!(!plan.contains("p-7"));
    assert!(plan.contains("其余步骤省略"));
}

#[test]
fn memory_is_truncated_before_it_can_dominate_the_prompt() {
    let composition = composed(500);
    let memory = section(&composition.system, "长期记忆");
    assert!(!memory.is_empty());
    assert!(
        estimate_text_tokens(&memory) <= MEMORY_BUDGET + SLACK,
        "memory section: {} tokens",
        estimate_text_tokens(&memory)
    );
    assert!(composition.memory_dropped > 100);
}
