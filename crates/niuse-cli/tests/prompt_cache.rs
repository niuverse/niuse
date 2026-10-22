//! Prompt-cache invariant: the stable prefix is everything before `## 今天`.
//!
//! Goals, plan, and memory may change between sessions; within a session they
//! are the cacheable prefix. Volatile sections must stay after it.

use niuse_core::prompt::compose_with_plan;
use niuse_core::{Date, Goal, JournalDay, LoopItem, MemoryEntry, MemoryKind, PlanStep};

fn today(days: i64) -> Date {
    Date::from_days(20_000 + days)
}

fn memory(text: &str) -> MemoryEntry {
    MemoryEntry::new(
        MemoryKind::Lesson,
        text,
        vec!["cache".into()],
        today(0),
        None,
    )
}

fn compose(
    memories: &[MemoryEntry],
    goals: &[Goal],
    plan: &[PlanStep],
    day: i64,
    loop_text: &str,
    journal_text: &str,
) -> String {
    compose_with_plan(
        Some("# 你是 niubot\n短句。"),
        Some("Rust 工程师。"),
        goals,
        plan,
        memories,
        &[LoopItem::new(loop_text, Some(today(day)))],
        &[JournalDay {
            date: today(day),
            bullets: vec![journal_text.into()],
        }],
        today(day),
    )
    .system
}

fn stable_prefix(system: &str) -> &str {
    system
        .split("## 今天")
        .next()
        .expect("today marker is always present")
}

#[test]
fn volatile_sections_do_not_change_the_cacheable_prefix() {
    let memories = [memory("prefer small patches")];
    let goals = [Goal::new("keep the cache warm", today(0))];
    let plan = [PlanStep::new(1, "inspect the prefix", today(0))];
    let first = compose(&memories, &goals, &plan, 0, "ship the tests", "decided A");
    let second = compose(&memories, &goals, &plan, 3, "call the vendor", "decided B");

    let a = stable_prefix(&first);
    let b = stable_prefix(&second);
    if a != b {
        let position = a
            .char_indices()
            .zip(b.char_indices())
            .find(|((_, left), (_, right))| left != right)
            .map(|((index, _), _)| index)
            .unwrap_or(a.len().min(b.len()));
        println!(
            "prefix differs at {position}: left={:?} right={:?}",
            &a[position..],
            &b[position..]
        );
    }
    assert_eq!(a, b);
    assert_ne!(first, second, "volatile sections should still differ");
    assert!(first.find("## 记忆协议") < first.find("## 今天"));
    assert_eq!(first.matches("## 记忆协议").count(), 1);
    assert!(first.find("## 今天") < first.find("## 未完成的承诺"));
    assert!(first.find("## 未完成的承诺") < first.find("## 最近几天"));
}

#[test]
fn a_memory_change_invalidates_the_prefix_on_purpose() {
    let goals = [Goal::new("keep the cache warm", today(0))];
    let plan = [PlanStep::new(1, "inspect the prefix", today(0))];
    let first = compose(
        &[memory("prefer small patches")],
        &goals,
        &plan,
        0,
        "ship",
        "A",
    );
    let second = compose(
        &[memory("prefer large rewrites")],
        &goals,
        &plan,
        0,
        "ship",
        "A",
    );
    assert_ne!(stable_prefix(&first), stable_prefix(&second));
}

#[test]
fn the_plan_stays_in_the_stable_prefix() {
    let system = compose(
        &[memory("prefer small patches")],
        &[Goal::new("keep the cache warm", today(0))],
        &[PlanStep::new(1, "inspect the prefix", today(0))],
        0,
        "ship",
        "A",
    );
    let prefix = stable_prefix(&system);
    assert!(prefix.contains("p-1 inspect the prefix"));
    assert!(prefix.contains("keep the cache warm"));
    assert!(prefix.contains("prefer small patches"));
}
