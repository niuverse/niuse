//! System-prompt assembly.
//!
//! Stable sections come first (persona, identity, user, memory, protocol) so the
//! provider can reuse its prompt cache; volatile sections (today, loops,
//! journal) come last. The whole prompt is built once per session.

use crate::date::Date;
use crate::model::{JournalDay, LoopItem, MemoryEntry};

pub const IDENTITY_BUDGET: usize = 600;
pub const USER_BUDGET: usize = 600;
pub const MEMORY_BUDGET: usize = 2_000;
pub const LOOP_BUDGET: usize = 300;
pub const JOURNAL_BUDGET: usize = 600;

pub const BASE: &str = "你是一个长期陪伴这位用户的工程伙伴。\n\
你的名字、语气与边界由后面的「你是谁」一节定义；本段只描述工作方式，产品名不是你的名字。\n\
你不是通用助手。你的价值不在「什么都能答」，而在三件事：记得他做过的判断、追踪你自己许下的承诺、在他判断错的时候开口。\n\
工作方式：\n\
- 具体优于笼统。引用过去的决定时说清楚是哪一次、为什么，不要泛泛地说「你喜欢简洁」。\n\
- 不表演情绪，不用昵称轰炸，不主动抒情。\n\
- 不同意就直说，给出理由，对事不对人。\n\
- 不确定就去查；查不到就说不知道，不要补全。\n\
- 记忆是偏好，不是法律。与当前证据冲突时以证据为准，并指出你不同意哪一条。";

pub const PROTOCOL: &str = "## 记忆协议\n\
你可以在回答的最后输出以下块。运行时会解析它们，并从用户看到的文本里剥离；不要在正文里提到这些块。\n\n\
<<niuse:memory kind=preference tags=rust,testing>>\n\
一句能独立看懂、跨项目仍然成立的结论\n\
<</niuse:memory>>\n\n\
<<niuse:loop due=2026-10-22>>\n\
你答应过、或用户交代你下次要做的事\n\
<</niuse:loop>>\n\n\
<<niuse:journal>>\n\
- 今天发生的决定、转折、未解问题或情绪变化（最多 3 条）\n\
<</niuse:journal>>\n\n\
规则：\n\
- 每轮最多一个 memory 块；没有值得记的就不要输出，连续多轮没有候选是正常的。\n\
- 只记「跨会话仍然有用、在仓库里 grep 不出来、一句话说得清」的内容。\n\
- memory 写偏好、决策、踩坑；不要写代码事实、环境配置、别人的信息。\n\
- 绝不把密钥、token、密码、客户名写进任何块。\n\
- journal 只写与上次不同的东西；「今天修了个 bug」这种流水账不要写。";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composition {
    pub system: String,
    pub memory_injected: usize,
    pub memory_dropped: usize,
    pub tokens: usize,
}

/// Conservative token estimate: ASCII averages four characters per token,
/// every non-ASCII character counts as one. Matches niucode's estimator.
pub fn estimate_text_tokens(text: &str) -> usize {
    let mut ascii = 0usize;
    let mut other = 0usize;
    for character in text.chars() {
        if character.is_ascii() {
            ascii += 1;
        } else {
            other += 1;
        }
    }
    ascii.div_ceil(4) + other
}

pub fn compose(
    identity: Option<&str>,
    user: Option<&str>,
    memories: &[MemoryEntry],
    loops: &[LoopItem],
    journal: &[JournalDay],
    today: Date,
) -> Composition {
    let mut sections: Vec<(&str, String)> = Vec::new();

    if let Some(identity) = identity.map(str::trim).filter(|text| !text.is_empty()) {
        sections.push((
            "你是谁",
            format!(
                "以下是你名字、语气与边界的唯一真源；如果本 prompt 的其他段落（包括产品名）与本段冲突，以本段为准。\n\n{}",
                truncate_to_tokens(identity, IDENTITY_BUDGET)
            ),
        ));
    }
    if let Some(user) = user.map(str::trim).filter(|text| !text.is_empty()) {
        sections.push(("你在和谁一起工作", truncate_to_tokens(user, USER_BUDGET)));
    }

    let (memory_text, memory_injected, memory_dropped) = memory_section(memories, today);
    if !memory_text.is_empty() {
        sections.push((
            "长期记忆",
            format!(
                "以下是你和用户长期积累的判断。它们是偏好，不是法律；与当前证据冲突时以证据为准，并指出你不同意哪一条。\n\n{memory_text}"
            ),
        ));
    }

    // Constant: keeping it before the volatile sections lengthens the stable
    // cache prefix.
    sections.push(("记忆协议", PROTOCOL.to_string()));
    sections.push(("今天", today.format()));

    let loop_text = loop_section(loops, today);
    if !loop_text.is_empty() {
        sections.push(("未完成的承诺", loop_text));
    }

    let journal_text = journal_section(journal);
    if !journal_text.is_empty() {
        sections.push(("最近几天", journal_text));
    }

    let mut system = String::from(BASE);
    for (title, body) in &sections {
        system.push_str("\n\n## ");
        system.push_str(title);
        system.push('\n');
        system.push_str(body);
    }

    Composition {
        tokens: estimate_text_tokens(&system),
        system,
        memory_injected,
        memory_dropped,
    }
}

fn memory_section(memories: &[MemoryEntry], today: Date) -> (String, usize, usize) {
    let mut ordered: Vec<&MemoryEntry> = memories
        .iter()
        .filter(|entry| !entry.is_expired(today))
        .collect();
    ordered.sort_by_key(|entry| (entry.is_stale(today), std::cmp::Reverse(entry.confirmed)));

    let mut lines = String::new();
    let mut used = 0usize;
    let mut injected = 0usize;
    for entry in &ordered {
        let mut line = format!(
            "- [m-{} · {} · {}] {}",
            entry.id,
            if entry.tags.is_empty() {
                "general".to_string()
            } else {
                entry.tags.join(", ")
            },
            entry.confirmed,
            entry.text
        );
        if entry.is_stale(today) {
            line.push_str(&format!("（未验证，自 {} 起）", entry.confirmed));
        }
        let cost = estimate_text_tokens(&line) + 1;
        if used + cost > MEMORY_BUDGET {
            continue;
        }
        used += cost;
        injected += 1;
        lines.push_str(&line);
        lines.push('\n');
    }

    let dropped = memories.len().saturating_sub(injected);
    (lines.trim_end().to_string(), injected, dropped)
}

fn loop_section(loops: &[LoopItem], today: Date) -> String {
    let mut open: Vec<&LoopItem> = loops.iter().filter(|item| !item.done).collect();
    open.sort_by_key(|item| (item.due.is_none(), item.due, item.is_overdue(today)));

    let mut lines = String::new();
    let mut used = 0usize;
    for item in open {
        let mut line = format!("- [ ] l-{} {}", item.id, item.text);
        if let Some(due) = item.due {
            if item.is_overdue(today) {
                line.push_str(&format!("（截止 {due}，已逾期）"));
            } else {
                line.push_str(&format!("（截止 {due}）"));
            }
        }
        let cost = estimate_text_tokens(&line) + 1;
        if used + cost > LOOP_BUDGET {
            continue;
        }
        used += cost;
        lines.push_str(&line);
        lines.push('\n');
    }
    lines.trim_end().to_string()
}

fn journal_section(journal: &[JournalDay]) -> String {
    let mut chosen: Vec<&JournalDay> = Vec::new();
    let mut used = 0usize;
    for day in journal {
        let mut block = format!("### {}\n", day.date);
        for bullet in &day.bullets {
            block.push_str(&format!("- {bullet}\n"));
        }
        let cost = estimate_text_tokens(&block) + 1;
        if used + cost > JOURNAL_BUDGET {
            continue;
        }
        used += cost;
        chosen.push(day);
    }
    chosen.reverse();

    let mut text = String::new();
    for day in chosen {
        text.push_str(&format!("### {}\n", day.date));
        for bullet in &day.bullets {
            text.push_str(&format!("- {bullet}\n"));
        }
        text.push('\n');
    }
    text.trim_end().to_string()
}

fn truncate_to_tokens(text: &str, budget: usize) -> String {
    let mut ascii = 0usize;
    let mut other = 0usize;
    let mut end = 0usize;
    for (index, character) in text.char_indices() {
        if character.is_ascii() {
            ascii += 1;
        } else {
            other += 1;
        }
        if ascii.div_ceil(4) + other > budget {
            break;
        }
        end = index + character.len_utf8();
    }
    if end >= text.len() {
        return text.to_string();
    }
    format!("{}\n……（已截断）", &text[..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{LoopItem, MemoryKind};

    fn date(input: &str) -> Date {
        Date::parse(input).unwrap()
    }

    fn entry(text: &str, confirmed: &str) -> MemoryEntry {
        MemoryEntry {
            id: "abcd".into(),
            kind: MemoryKind::Preference,
            text: text.into(),
            tags: vec!["rust".into()],
            created: date(confirmed),
            confirmed: date(confirmed),
            source: None,
        }
    }

    #[test]
    fn order_is_stable_before_volatile() {
        let today = date("2026-10-21");
        let memory = entry("先给结论", "2026-10-20");
        let loop_item = LoopItem::new("查 cwd", Some(date("2026-10-22")));
        let journal = vec![JournalDay {
            date: today,
            bullets: vec!["决定 A".into()],
        }];
        let composition = compose(
            Some("你是 Niuse。"),
            Some("用户在做机器人。"),
            &[memory],
            &[loop_item],
            &journal,
            today,
        );
        let system = composition.system;
        let identity = system.find("你是谁").unwrap();
        let user = system.find("你在和谁一起工作").unwrap();
        let memory = system.find("长期记忆").unwrap();
        let protocol = system.find("记忆协议").unwrap();
        let loops = system.find("未完成的承诺").unwrap();
        let journal = system.find("最近几天").unwrap();
        assert!(identity < user);
        assert!(user < memory);
        assert!(memory < protocol);
        assert!(protocol < loops);
        assert!(loops < journal);
        assert_eq!(composition.memory_injected, 1);
    }

    #[test]
    fn identity_is_the_only_name_source() {
        assert!(!BASE.contains("你是 niuse"));
        let composition = compose(
            Some("# 你是 niubot"),
            None,
            &[],
            &[],
            &[],
            date("2026-10-22"),
        );
        assert!(composition.system.contains("niubot"));
        assert!(composition.system.contains("唯一真源"));
        assert!(composition.system.contains("产品名不是你的名字"));
    }

    #[test]
    fn expired_memory_is_dropped_and_stale_is_labelled() {
        let today = date("2026-10-21");
        let fresh = entry("新结论", "2026-10-20");
        let stale = entry("三个月前的结论", "2026-08-01");
        let expired = entry("很久以前的结论", "2026-06-01");
        let composition = compose(None, None, &[fresh, stale, expired], &[], &[], today);
        assert!(composition.system.contains("新结论"));
        assert!(composition.system.contains("未验证，自 2026-08-01 起"));
        assert!(!composition.system.contains("很久以前的结论"));
        assert_eq!(composition.memory_injected, 2);
        assert_eq!(composition.memory_dropped, 1);
    }

    #[test]
    fn oversized_journal_day_does_not_hide_older_ones() {
        let today = date("2026-10-21");
        let huge = JournalDay {
            date: today,
            bullets: vec!["x".repeat(4_000)],
        };
        let small = JournalDay {
            date: date("2026-10-20"),
            bullets: vec!["pay the invoice".into()],
        };
        let composition = compose(None, None, &[], &[], &[huge, small], today);
        assert!(composition.system.contains("pay the invoice"));
        assert!(!composition.system.contains("xxxx"));
    }

    #[test]
    fn oversized_promise_does_not_hide_smaller_ones() {
        let today = date("2026-10-21");
        let huge = LoopItem::new("x".repeat(2_000), Some(date("2026-10-01")));
        let small = LoopItem::new("pay the invoice", None);
        let composition = compose(None, None, &[], &[huge, small], &[], today);
        assert!(composition.system.contains("pay the invoice"));
        assert!(!composition.system.contains("xxxx"));
    }

    #[test]
    fn memory_budget_holds_at_capacity() {
        let today = date("2026-10-21");
        let memories: Vec<MemoryEntry> = (0..250)
            .map(|index| {
                entry(
                    &format!(
                        "preference number {index} with enough words to consume the prompt budget quickly"
                    ),
                    "2026-10-20",
                )
            })
            .collect();
        let composition = compose(None, None, &memories, &[], &[], today);
        assert_eq!(
            composition.memory_injected + composition.memory_dropped,
            250
        );
        assert!(composition.memory_injected > 0);
        assert!(
            composition.memory_injected < 250,
            "the budget should drop entries once the store is large"
        );
        assert!(
            composition.tokens < 6_000,
            "system prompt grew to {} tokens",
            composition.tokens
        );
    }

    #[test]
    fn long_identity_is_truncated() {
        let today = date("2026-10-21");
        let identity = "很长的身份。".repeat(400);
        let composition = compose(Some(&identity), None, &[], &[], &[], today);
        assert!(composition.system.contains("已截断"));
        assert!(estimate_text_tokens(&composition.system) < 1_500);
    }

    #[test]
    fn empty_sections_are_omitted() {
        let today = date("2026-10-21");
        let composition = compose(None, None, &[], &[], &[], today);
        assert!(!composition.system.contains("长期记忆"));
        assert!(!composition.system.contains("未完成的承诺"));
        assert!(!composition.system.contains("最近几天"));
        assert!(composition.system.contains("记忆协议"));
    }
}
