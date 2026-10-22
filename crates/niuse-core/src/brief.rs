//! A short "what needs attention" view assembled from the five stores.
//!
//! The brief is derived, read-only, and provider-free: it never creates a
//! store, keeps no state, and can be tested without a terminal.

use anyhow::Result;

use crate::date::Date;
use crate::model::LoopItem;
use crate::store::Store;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Brief {
    pub today: Date,
    pub project: String,
    pub has_identity: bool,
    pub has_user: bool,
    pub memories: usize,
    pub stale: usize,
    pub expired: usize,
    pub open_loops: usize,
    pub open_goals: usize,
    pub overdue: Vec<LoopItem>,
    pub inbox: usize,
    pub journal_last: Option<Date>,
}

impl Brief {
    /// Read the stores and build the view. The project label is supplied by the
    /// caller because the store does not know the working directory.
    pub fn build(store: &Store, today: Date, project: impl Into<String>) -> Result<Self> {
        let memories = store.memories()?;
        let stale = memories
            .iter()
            .filter(|entry| entry.is_stale(today) && !entry.is_expired(today))
            .count();
        let expired = memories
            .iter()
            .filter(|entry| entry.is_expired(today))
            .count();

        let loops = store.loops()?;
        let open_loops = loops.iter().filter(|item| !item.done).count();
        let open_goals = store.goals()?.iter().filter(|goal| !goal.done).count();
        let mut overdue: Vec<LoopItem> = loops
            .into_iter()
            .filter(|item| item.is_overdue(today))
            .collect();
        overdue.sort_by(|left, right| {
            left.due
                .cmp(&right.due)
                .then_with(|| left.id.cmp(&right.id))
        });

        let inbox = store
            .read_inbox()?
            .unwrap_or_default()
            .lines()
            .filter(|line| line.trim_start().starts_with("- [ ] "))
            .count();
        let journal_last = store.recent_journal(1)?.first().map(|day| day.date);
        let project = project.into();
        let project = if project.trim().is_empty() {
            "unknown".to_string()
        } else {
            project
        };

        Ok(Self {
            today,
            project,
            has_identity: store.identity()?.is_some(),
            has_user: store.user()?.is_some(),
            memories: memories.len(),
            stale,
            expired,
            open_loops,
            open_goals,
            overdue,
            inbox,
            journal_last,
        })
    }

    /// Whether there is something the human should act on at session start.
    pub fn needs_attention(&self) -> bool {
        !self.overdue.is_empty() || self.inbox > 0 || self.expired > 0 || self.stale > 0
    }

    /// A compact block for the interactive banner. Empty when nothing is due.
    pub fn startup_lines(&self) -> Vec<String> {
        let mut parts = Vec::new();
        if !self.overdue.is_empty() {
            parts.push(format!("{} 条逾期承诺", self.overdue.len()));
        }
        if self.inbox > 0 {
            parts.push(format!("{} 条待审查", self.inbox));
        }
        if self.expired > 0 {
            parts.push(format!("{} 条记忆已过期", self.expired));
        }
        if self.stale > 0 {
            parts.push(format!("{} 条记忆待验证", self.stale));
        }
        if parts.is_empty() {
            return Vec::new();
        }

        let mut lines = vec![format!("须处理：{}", parts.join(" · "))];
        for item in self.overdue.iter().take(3) {
            lines.push(format!("  {}", item.render().trim_start_matches("- [ ] ")));
        }
        if self.overdue.len() > 3 {
            lines.push(format!("  …还有 {} 条逾期承诺", self.overdue.len() - 3));
        }
        lines
    }

    /// The full view used by `niuse brief` and `/brief`.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "niuse brief · {} · {}\n",
            self.today, self.project
        ));
        out.push_str(&format!(
            "identity   {}\n",
            if self.has_identity {
                "已设置"
            } else {
                "未设置"
            }
        ));
        out.push_str(&format!(
            "user       {}\n",
            if self.has_user {
                "已设置"
            } else {
                "未设置"
            }
        ));
        out.push_str(&format!(
            "memory     {} 条（{} 条待验证，{} 条已过期）\n",
            self.memories, self.stale, self.expired
        ));
        if self.open_loops == 0 {
            out.push_str("loops      没有未完成承诺\n");
        } else if self.overdue.is_empty() {
            out.push_str(&format!("loops      {} 条未完成\n", self.open_loops));
        } else {
            out.push_str(&format!(
                "loops      {} 条未完成，{} 条逾期\n",
                self.open_loops,
                self.overdue.len()
            ));
        }
        for item in &self.overdue {
            out.push_str(&format!(
                "  {}\n",
                item.render().trim_start_matches("- [ ] ")
            ));
        }
        if self.open_goals == 0 {
            out.push_str("goals      没有进行中的目标\n");
        } else {
            out.push_str(&format!("goals      {} 条进行中\n", self.open_goals));
        }
        if self.inbox > 0 {
            out.push_str(&format!(
                "inbox      {} 条待审查（niuse review）\n",
                self.inbox
            ));
        }
        match self.journal_last {
            Some(date) => {
                let when = match date.age_days(self.today) {
                    0 => "今天".to_string(),
                    1 => "昨天".to_string(),
                    days => format!("{days} 天前"),
                };
                out.push_str(&format!("journal    最近 {}（{}）\n", date, when));
            }
            None => out.push_str("journal    还没有记录\n"),
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MemoryEntry, MemoryKind};

    fn temp_store() -> (tempfile::TempDir, Store) {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        (directory, store)
    }

    #[test]
    fn empty_store_is_quiet() {
        let (_directory, store) = temp_store();
        let brief = Brief::build(&store, Date::today(), "niuse").unwrap();
        assert!(!brief.needs_attention());
        assert!(brief.startup_lines().is_empty());
        let rendered = brief.render();
        assert!(rendered.contains("还没有记录"));
        assert!(rendered.contains("没有未完成承诺"));
    }

    #[test]
    fn overdue_expired_and_inbox_are_actionable() {
        let (_directory, store) = temp_store();
        let today = Date::today();

        let mut stale = MemoryEntry::new(MemoryKind::Preference, "stale", vec![], today, None);
        stale.confirmed = today.plus_days(-40);
        store.add_memory(stale).unwrap();
        let mut expired = MemoryEntry::new(MemoryKind::Lesson, "expired", vec![], today, None);
        expired.confirmed = today.plus_days(-100);
        store.add_memory(expired).unwrap();

        let due = today.plus_days(-2);
        store
            .add_loop(LoopItem::new("ship the brief", Some(due)))
            .unwrap();
        store.add_loop(LoopItem::new("no deadline", None)).unwrap();
        store
            .append_journal(today.plus_days(-1), &["did the work".into()])
            .unwrap();
        store.write_inbox("- [ ] pending\n- [x] done\n").unwrap();

        let brief = Brief::build(&store, today, "niuse").unwrap();
        assert_eq!(brief.memories, 2);
        assert_eq!(brief.stale, 1);
        assert_eq!(brief.expired, 1);
        assert_eq!(brief.open_loops, 2);
        assert_eq!(brief.overdue.len(), 1);
        assert_eq!(brief.inbox, 1);
        assert_eq!(brief.journal_last, Some(today.plus_days(-1)));
        assert!(brief.needs_attention());

        let lines = brief.startup_lines();
        assert!(lines[0].contains("1 条逾期承诺"));
        assert!(lines[0].contains("1 条待审查"));
        assert!(lines.iter().any(|line| line.contains("ship the brief")));
        let rendered = brief.render();
        assert!(rendered.contains("2 条（1 条待验证，1 条已过期）"));
        assert!(rendered.contains("昨天"));
    }

    #[test]
    fn overdue_is_sorted_and_capped() {
        let (_directory, store) = temp_store();
        let today = Date::today();
        for (index, days) in [2, 5, 1, 9].into_iter().enumerate() {
            let text = format!("promise {index}");
            store
                .add_loop(LoopItem::new(text, Some(today.plus_days(-days))))
                .unwrap();
        }

        let brief = Brief::build(&store, today, "niuse").unwrap();
        assert_eq!(brief.overdue.len(), 4);
        assert_eq!(brief.overdue[0].due, Some(today.plus_days(-9)));
        let lines = brief.startup_lines();
        assert_eq!(lines.len(), 5);
        assert!(lines.last().unwrap().contains("还有 1 条"));
    }

    #[test]
    fn blank_project_falls_back_to_unknown() {
        let (_directory, store) = temp_store();
        let brief = Brief::build(&store, Date::today(), "  ").unwrap();
        assert_eq!(brief.project, "unknown");
    }
}
