//! Journal commands: list days, search bullets, and add a manual entry.

use anyhow::{bail, Result};
use niuse_core::model::one_line;
use niuse_core::policy::{search_score, secret_reason};
use niuse_core::{Date, JournalDay, Store};

#[derive(Debug, serde::Serialize)]
struct JournalDayJson {
    date: String,
    bullets: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct JournalHitJson {
    date: String,
    text: String,
}

pub fn journal_show(store: &Store, days: usize, json: bool) -> Result<()> {
    let days = store.recent_journal(days)?;
    if json {
        let items: Vec<JournalDayJson> = days
            .iter()
            .map(|day| JournalDayJson {
                date: day.date.to_string(),
                bullets: day.bullets.clone(),
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }
    if days.is_empty() {
        println!("（还没有 journal。）");
        return Ok(());
    }
    for day in days {
        println!("## {}", day.date);
        for bullet in &day.bullets {
            println!("- {bullet}");
        }
        println!();
    }
    Ok(())
}

pub fn journal_add(store: &Store, text: &str) -> Result<()> {
    let text = one_line(text.trim().to_string());
    if text.is_empty() {
        bail!("journal text is empty");
    }
    if let Some(reason) = secret_reason(&text) {
        bail!("refusing to store: looks like a {reason}");
    }
    if store.append_journal(Date::today(), std::slice::from_ref(&text))? {
        if let Err(error) = store.append_metric("journal", "1") {
            eprintln!("warning: cannot record metric: {error:#}");
        }
        println!("journal +1");
    }
    Ok(())
}

pub fn journal_search(store: &Store, query: &str, json: bool) -> Result<()> {
    let query = query.trim();
    if query.is_empty() {
        bail!("usage: niuse journal --search <query>");
    }
    let days = store.recent_journal(usize::MAX)?;
    let hits = rank_journal(query, &days);
    if json {
        let items: Vec<JournalHitJson> = hits
            .iter()
            .map(|(_, day, text)| JournalHitJson {
                date: day.date.to_string(),
                text: (*text).to_string(),
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }
    if hits.is_empty() {
        println!("没有匹配「{query}」的日记。");
        return Ok(());
    }
    println!("{} 条匹配「{query}」", hits.len());
    for (_, day, bullet) in hits.iter().take(10) {
        println!("{}  {bullet}", day.date);
    }
    Ok(())
}

fn rank_journal<'a>(query: &str, days: &'a [JournalDay]) -> Vec<(f64, &'a JournalDay, &'a str)> {
    let mut hits = Vec::new();
    for day in days {
        for bullet in &day.bullets {
            let score = search_score(query, bullet, &[]);
            if score >= 0.2 {
                hits.push((score, day, bullet.as_str()));
            }
        }
    }
    hits.sort_by(|left, right| {
        right
            .0
            .partial_cmp(&left.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_journal_entry_persists() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        journal_add(&store, "decided to ship the config file").unwrap();
        let days = store.recent_journal(1).unwrap();
        assert_eq!(days.len(), 1);
        assert!(days[0]
            .bullets
            .iter()
            .any(|bullet| bullet.contains("config file")));
    }

    #[test]
    fn journal_search_ranks_substring_hits() {
        let day = JournalDay {
            date: Date::parse("2026-10-21").unwrap(),
            bullets: vec![
                "decided to use cargo nextest".into(),
                "unrelated thought about lunch".into(),
            ],
        };
        let days = vec![day];
        let hits = rank_journal("nextest", &days);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].2, "decided to use cargo nextest");
    }

    #[test]
    fn journal_search_returns_nothing_for_unrelated_queries() {
        let day = JournalDay {
            date: Date::parse("2026-10-21").unwrap(),
            bullets: vec!["decided to use cargo nextest".into()],
        };
        assert!(rank_journal("kubernetes", &[day]).is_empty());
    }
}
