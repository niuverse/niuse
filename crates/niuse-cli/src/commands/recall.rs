//! Unified, bounded recall across memory, journal, and loops.
//!
//! This is the human-facing half of on-demand recall: it searches expired
//! memory that is no longer injected, old journal days, and promises. It is
//! read-only and keeps no index.

use anyhow::{bail, Result};
use niuse_core::policy::search_score;
use niuse_core::{Date, Store};
use serde::Serialize;

const THRESHOLD: f64 = 0.2;

#[derive(Debug, Serialize)]
struct Hit {
    kind: &'static str,
    id: String,
    date: Option<String>,
    detail: String,
    text: String,
    score: f64,
}

pub fn recall(store: &Store, query: &str, limit: usize, json: bool) -> Result<()> {
    let hits = collect(store, query, limit)?;
    if json {
        let payload = serde_json::json!({
            "query": query.trim(),
            "hits": hits,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    print!("{}", render(query, &hits));
    Ok(())
}

/// Rendered form for the REPL/TUI, where the host owns output.
pub fn recall_text(store: &Store, query: &str, limit: usize) -> Result<String> {
    let hits = collect(store, query, limit)?;
    Ok(render(query, &hits))
}

fn collect(store: &Store, query: &str, limit: usize) -> Result<Vec<Hit>> {
    let query = query.trim();
    if query.is_empty() {
        bail!("usage: niuse recall <query>");
    }
    let limit = limit.clamp(1, 100);
    let mut hits = Vec::new();

    let today = Date::today();
    for entry in store.memories()? {
        let score = search_score(query, &entry.text, &entry.tags);
        if score < THRESHOLD {
            continue;
        }
        let mut detail = entry.kind.as_str().to_string();
        if !entry.tags.is_empty() {
            detail.push_str(&format!(" · tags={}", entry.tags.join(",")));
        }
        if entry.is_expired(today) {
            detail.push_str(" · expired");
        } else if entry.is_stale(today) {
            detail.push_str(" · unverified");
        }
        hits.push(Hit {
            kind: "memory",
            id: format!("m-{}", entry.id),
            date: Some(entry.confirmed.format()),
            detail,
            text: entry.text,
            score,
        });
    }

    for day in store.recent_journal(usize::MAX)? {
        for bullet in day.bullets {
            let score = search_score(query, &bullet, &[]);
            if score < THRESHOLD {
                continue;
            }
            hits.push(Hit {
                kind: "journal",
                id: day.date.format(),
                date: Some(day.date.format()),
                detail: "journal".to_string(),
                text: bullet,
                score,
            });
        }
    }

    for item in store.loops()? {
        let score = search_score(query, &item.text, &[]);
        if score < THRESHOLD {
            continue;
        }
        let mut detail = if item.done { "done" } else { "open" }.to_string();
        if let Some(due) = item.due {
            detail.push_str(&format!(" · due {due}"));
        }
        hits.push(Hit {
            kind: "loop",
            id: format!("l-{}", item.id),
            date: item.due.map(|due| due.format()),
            detail,
            text: item.text,
            score,
        });
    }

    for goal in store.goals()? {
        let score = search_score(query, &goal.text, &[]);
        if score < THRESHOLD {
            continue;
        }
        hits.push(Hit {
            kind: "goal",
            id: format!("g-{}", goal.id),
            date: Some(goal.created.format()),
            detail: if goal.done { "done" } else { "open" }.to_string(),
            text: goal.text,
            score,
        });
    }

    hits.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.kind.cmp(right.kind))
            .then_with(|| left.id.cmp(&right.id))
    });
    hits.truncate(limit);
    Ok(hits)
}

fn render(query: &str, hits: &[Hit]) -> String {
    let mut out = format!("recall \"{}\" · {} 条\n", query.trim(), hits.len());
    if hits.is_empty() {
        out.push_str("没有命中。可以换一个更短的关键词。\n");
        return out;
    }
    for hit in hits {
        let date = hit.date.as_deref().unwrap_or("-");
        out.push_str(&format!(
            "[{}] {} {} · {}\n  {}\n",
            hit.kind, hit.id, date, hit.detail, hit.text
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use niuse_core::{Goal, MemoryEntry, MemoryKind};

    fn temp_store() -> (tempfile::TempDir, Store) {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        (directory, store)
    }

    fn memory(store: &Store, text: &str, confirmed: Date) {
        let mut entry = MemoryEntry::new(
            MemoryKind::Lesson,
            text,
            vec!["rust".to_string()],
            Date::today(),
            None,
        );
        entry.confirmed = confirmed;
        store.add_memory(entry).unwrap();
    }

    #[test]
    fn empty_store_reports_a_miss() {
        let (_directory, store) = temp_store();
        let text = recall_text(&store, "rust", 10).unwrap();
        assert!(text.contains("没有命中"));
    }

    #[test]
    fn limit_caps_hits_and_expired_memory_is_labelled() {
        let (_directory, store) = temp_store();
        let today = Date::today();
        memory(&store, "rust one", today);
        memory(&store, "rust two", today);
        memory(&store, "rust expired", today.plus_days(-100));

        let hits = collect(&store, "rust", 1).unwrap();
        assert_eq!(hits.len(), 1);

        let all = collect(&store, "rust", 10).unwrap();
        assert_eq!(all.len(), 3);
        assert!(all
            .iter()
            .any(|hit| hit.text == "rust expired" && hit.detail.contains("expired")));

        let goal = store.add_goal(Goal::new("rust goal", today)).unwrap();
        let all = collect(&store, "rust", 10).unwrap();
        assert!(all
            .iter()
            .any(|hit| hit.kind == "goal" && hit.id == format!("g-{}", goal.id)));
    }
}
