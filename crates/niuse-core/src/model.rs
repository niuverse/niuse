//! The durable records niuse keeps about a person and a working relationship:
//! memory entries, open loops, and journal days.
//!
//! Every record renders to Markdown that a human can edit. Parsing is tolerant:
//! unknown lines are ignored, and a broken field costs one record, not the file.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::date::Date;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    Preference,
    Lesson,
}

impl MemoryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Preference => "preference",
            Self::Lesson => "lesson",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Preference => "偏好",
            Self::Lesson => "决策与踩坑",
        }
    }

    pub fn file(self) -> &'static str {
        match self {
            Self::Preference => "preferences.md",
            Self::Lesson => "lessons.md",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_lowercase().as_str() {
            "preference" | "pref" | "偏好" => Some(Self::Preference),
            "lesson" | "decision" | "pitfall" | "决策" | "踩坑" => Some(Self::Lesson),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryEntry {
    pub id: String,
    pub kind: MemoryKind,
    pub text: String,
    pub tags: Vec<String>,
    pub created: Date,
    pub confirmed: Date,
    pub source: Option<String>,
}

impl MemoryEntry {
    pub fn new(
        kind: MemoryKind,
        text: impl Into<String>,
        tags: Vec<String>,
        today: Date,
        source: Option<String>,
    ) -> Self {
        let text = one_line(text.into());
        Self {
            id: new_id(&text),
            kind,
            text,
            tags: normalize_tags(tags),
            created: today,
            confirmed: today,
            source,
        }
    }

    pub fn age_days(&self, today: Date) -> i64 {
        self.confirmed.age_days(today)
    }

    /// Older than a month: still injected, but labelled so the model verifies
    /// before relying on it.
    pub fn is_stale(&self, today: Date) -> bool {
        self.age_days(today) > 30
    }

    /// Older than a quarter: kept on disk, no longer injected automatically.
    pub fn is_expired(&self, today: Date) -> bool {
        self.age_days(today) > 90
    }

    pub fn render(&self) -> String {
        let mut out = format!(
            "### m-{}\n- 结论：{}\n- 类型：{}\n",
            self.id,
            self.text,
            self.kind.as_str()
        );
        if !self.tags.is_empty() {
            out.push_str(&format!("- 标签：{}\n", self.tags.join(", ")));
        }
        out.push_str(&format!("- 创建：{}\n", self.created));
        out.push_str(&format!("- 最后确认：{}\n", self.confirmed));
        if let Some(source) = &self.source {
            out.push_str(&format!("- 来源：{source}\n"));
        }
        out
    }
}

/// Parse one `### m-xxxx` section. Returns `None` when the section has no text.
pub fn parse_memory_section(
    section: &str,
    fallback: MemoryKind,
    today: Date,
) -> Option<MemoryEntry> {
    let mut lines = section.lines();
    let heading = lines.next()?.trim();
    let id = heading
        .strip_prefix("### m-")
        .map(str::trim)
        .filter(|id| !id.is_empty())?
        .to_string();

    let mut text = None;
    let mut kind = fallback;
    let mut tags = Vec::new();
    let mut created = today;
    let mut confirmed = today;
    let mut source = None;

    for line in lines {
        let Some((key, value)) = split_field(line) else {
            continue;
        };
        match key {
            "结论" | "text" => text = Some(value.to_string()),
            "类型" | "kind" => {
                if let Some(parsed) = MemoryKind::parse(value) {
                    kind = parsed;
                }
            }
            "标签" | "tags" => {
                tags = value
                    .split(',')
                    .map(str::trim)
                    .filter(|tag| !tag.is_empty())
                    .map(str::to_string)
                    .collect()
            }
            "创建" | "created" => created = Date::parse(value).unwrap_or(today),
            "最后确认" | "confirmed" => confirmed = Date::parse(value).unwrap_or(today),
            "来源" | "source" => source = Some(value.trim_matches('`').to_string()),
            _ => {}
        }
    }

    Some(MemoryEntry {
        id,
        kind,
        text: one_line(text?),
        tags,
        created,
        confirmed,
        source,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopItem {
    pub id: String,
    pub text: String,
    pub due: Option<Date>,
    pub done: bool,
}

impl LoopItem {
    pub fn new(text: impl Into<String>, due: Option<Date>) -> Self {
        let text = one_line(text.into());
        Self {
            id: new_id(&text),
            text,
            due,
            done: false,
        }
    }

    pub fn is_overdue(&self, today: Date) -> bool {
        self.due.is_some_and(|due| !self.done && due < today)
    }

    pub fn render(&self) -> String {
        let mark = if self.done { "x" } else { " " };
        match self.due {
            Some(due) => format!("- [{mark}] l-{} {}（截止 {due}）", self.id, self.text),
            None => format!("- [{mark}] l-{} {}", self.id, self.text),
        }
    }
}

pub fn parse_loop_line(line: &str) -> Option<LoopItem> {
    let rest = line.strip_prefix("- [")?;
    let mut chars = rest.chars();
    let mark = chars.next()?;
    if mark != ' ' && mark != 'x' {
        return None;
    }
    let rest = chars.as_str().strip_prefix("] ")?;
    let rest = rest.strip_prefix("l-")?;
    let (id, rest) = rest.split_once(char::is_whitespace)?;
    let rest = rest.trim();

    let mut text = rest.trim();
    let mut due = None;
    if let Some(open) = rest.rfind("（截止 ") {
        let tail = &rest[open + "（截止 ".len()..];
        if let Some(close) = tail.rfind('）') {
            due = Date::parse(&tail[..close]);
            if due.is_some() {
                text = rest[..open].trim();
            }
        }
    }

    Some(LoopItem {
        id: id.to_string(),
        text: text.to_string(),
        due,
        done: mark == 'x',
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Goal {
    pub id: String,
    pub text: String,
    pub created: Date,
    pub done: bool,
}

impl Goal {
    pub fn new(text: impl Into<String>, created: Date) -> Self {
        let text = one_line(text.into());
        Self {
            id: new_id(&text),
            text,
            created,
            done: false,
        }
    }

    pub fn render(&self) -> String {
        let mark = if self.done { "x" } else { " " };
        format!(
            "- [{mark}] g-{} {}（创建 {}）",
            self.id, self.text, self.created
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanStep {
    pub id: u32,
    pub text: String,
    pub created: Date,
    pub done: bool,
}

impl PlanStep {
    pub fn new(id: u32, text: impl Into<String>, created: Date) -> Self {
        Self {
            id,
            text: one_line(text.into()),
            created,
            done: false,
        }
    }

    pub fn label(&self) -> String {
        format!("p-{}", self.id)
    }

    pub fn render(&self) -> String {
        let mark = if self.done { "x" } else { " " };
        format!(
            "- [{mark}] p-{} {}（创建 {}）",
            self.id, self.text, self.created
        )
    }
}

pub fn parse_plan_line(line: &str) -> Option<PlanStep> {
    let rest = line.strip_prefix("- [")?;
    let mut chars = rest.chars();
    let mark = chars.next()?;
    if mark != ' ' && mark != 'x' {
        return None;
    }
    let rest = chars.as_str().strip_prefix("] p-")?;
    let (id, rest) = rest.split_once(char::is_whitespace)?;
    let id: u32 = id.parse().ok()?;
    let rest = rest.trim();

    let mut text = rest.trim();
    let mut created = Date::from_days(0);
    if let Some(open) = rest.rfind("（创建 ") {
        let tail = &rest[open + "（创建 ".len()..];
        if let Some(close) = tail.rfind('）') {
            if let Some(date) = Date::parse(&tail[..close]) {
                created = date;
                text = rest[..open].trim();
            }
        }
    }

    Some(PlanStep {
        id,
        text: text.to_string(),
        created,
        done: mark == 'x',
    })
}

pub fn parse_plan(text: &str) -> Vec<PlanStep> {
    text.lines().filter_map(parse_plan_line).collect()
}

pub fn parse_goal_line(line: &str) -> Option<Goal> {
    let rest = line.strip_prefix("- [")?;
    let mut chars = rest.chars();
    let mark = chars.next()?;
    if mark != ' ' && mark != 'x' {
        return None;
    }
    let rest = chars.as_str().strip_prefix("] ")?;
    let rest = rest.strip_prefix("g-")?;
    let (id, rest) = rest.split_once(char::is_whitespace)?;
    let rest = rest.trim();

    let mut text = rest.trim();
    let mut created = Date::from_days(0);
    if let Some(open) = rest.rfind("（创建 ") {
        let tail = &rest[open + "（创建 ".len()..];
        if let Some(close) = tail.rfind('）') {
            if let Some(date) = Date::parse(&tail[..close]) {
                created = date;
                text = rest[..open].trim();
            }
        }
    }

    Some(Goal {
        id: id.to_string(),
        text: text.to_string(),
        created,
        done: mark == 'x',
    })
}

pub fn parse_goals(text: &str) -> Vec<Goal> {
    text.lines().filter_map(parse_goal_line).collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalDay {
    pub date: Date,
    pub bullets: Vec<String>,
}

impl JournalDay {
    pub fn render(&self) -> String {
        let mut out = format!("## {}\n", self.date);
        for bullet in &self.bullets {
            out.push_str(&format!("- {bullet}\n"));
        }
        out
    }
}

/// Parse `## YYYY-MM-DD` day sections. Unparseable headings are skipped.
pub fn parse_journal(text: &str) -> Vec<JournalDay> {
    let mut days: Vec<JournalDay> = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        if let Some(heading) = line.strip_prefix("## ") {
            if let Some(date) = Date::parse(heading) {
                days.push(JournalDay {
                    date,
                    bullets: Vec::new(),
                });
            }
            continue;
        }
        if let Some(bullet) = line.strip_prefix("- ") {
            if let Some(day) = days.last_mut() {
                let bullet = one_line(bullet.to_string());
                if !bullet.is_empty() {
                    day.bullets.push(bullet);
                }
            }
        }
    }
    days
}

/// Four hex digits derived from the text and the clock. Collisions are checked
/// by the store, which retries with a fresh seed.
pub fn new_id(seed: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    nanos.hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    format!("{:04x}", hasher.finish() & 0xffff)
}

/// Keep records single-line: the file format is line-oriented.
pub fn one_line(input: String) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for tag in tags {
        let tag = tag.trim().trim_start_matches('#').to_lowercase();
        if !tag.is_empty() && !out.iter().any(|existing| existing == &tag) {
            out.push(tag);
        }
    }
    out
}

fn split_field(line: &str) -> Option<(&str, &str)> {
    let line = line.trim().strip_prefix("- ")?;
    let index = line
        .char_indices()
        .find(|(_, character)| *character == '：' || *character == ':')
        .map(|(index, _)| index)?;
    let key = line[..index].trim();
    let value = line[index..].trim_start_matches(['：', ':']).trim();
    Some((key, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(input: &str) -> Date {
        Date::parse(input).unwrap()
    }

    #[test]
    fn memory_round_trips_through_markdown() {
        let entry = MemoryEntry::new(
            MemoryKind::Lesson,
            "Rust 仓库一律用 cargo nextest\n不要用 cargo test",
            vec!["rust".into(), "Testing".into()],
            date("2026-10-21"),
            Some("session-1".into()),
        );
        assert_eq!(
            entry.text,
            "Rust 仓库一律用 cargo nextest 不要用 cargo test"
        );
        assert_eq!(entry.tags, vec!["rust", "testing"]);

        let parsed = parse_memory_section(
            entry.render().trim(),
            MemoryKind::Preference,
            date("2026-10-22"),
        )
        .expect("section parses");
        assert_eq!(parsed, entry);
    }

    #[test]
    fn loop_round_trips() {
        let item = LoopItem::new("确认 per-turn cwd", Some(date("2026-10-22")));
        let parsed = parse_loop_line(&item.render()).expect("loop parses");
        assert_eq!(parsed, item);
    }

    #[test]
    fn loop_without_due_round_trips() {
        let item = LoopItem::new("补 bootstrap 采样上限", None);
        let parsed = parse_loop_line(&item.render()).expect("loop parses");
        assert_eq!(parsed, item);
    }

    #[test]
    fn done_loop_parses() {
        let mut item = LoopItem::new("写 README", None);
        item.done = true;
        let parsed = parse_loop_line(&item.render()).expect("loop parses");
        assert!(parsed.done);
    }

    #[test]
    fn plan_steps_parse_and_render() {
        let step = PlanStep::new(2, "write tests", Date::from_days(20_000));
        let line = step.render();
        assert!(line.starts_with("- [ ] p-2 write tests"));
        let parsed = parse_plan_line(&line).expect("step parses");
        assert_eq!(parsed, step);
        let mut done = parsed;
        done.done = true;
        let parsed = parse_plan_line(&done.render()).expect("done step parses");
        assert!(parsed.done);
        assert_eq!(parse_plan("header\n- [ ] p-1 a\n- [x] p-2 b\n").len(), 2);
    }

    #[test]
    fn journal_sections_parse() {
        let text = "## 2026-10-20\n\n- 决定了 A\n- 未解 B\n\n## 2026-10-21\n\n- 转折 C\n";
        let days = parse_journal(text);
        assert_eq!(days.len(), 2);
        assert_eq!(days[0].bullets, vec!["决定了 A", "未解 B"]);
        assert_eq!(days[1].bullets, vec!["转折 C"]);
        assert_eq!(days[1].render(), "## 2026-10-21\n- 转折 C\n");
    }

    #[test]
    fn ids_are_four_hex_digits() {
        let id = new_id("hello");
        assert_eq!(id.len(), 4);
        assert!(id.chars().all(|character| character.is_ascii_hexdigit()));
    }

    #[test]
    fn tags_dedupe_and_lowercase() {
        assert_eq!(
            normalize_tags(vec![
                "Rust".into(),
                "#rust".into(),
                " ".into(),
                "test".into()
            ]),
            vec!["rust", "test"]
        );
    }
}
