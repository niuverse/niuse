//! Memory commands: list, search, add, show, edit, forget, confirm, review,
//! and prune.
//!
//! The store stays the source of truth; this module only shapes output and
//! applies write-time policy.

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use niuse_core::model::one_line;
use niuse_core::policy::{conflicts, search_score, secret_reason};
use niuse_core::{Date, MemoryEntry, MemoryKind, Store};

use super::{edit_text, normalize_id};

fn kind_summary(memories: &[MemoryEntry]) -> String {
    if memories.is_empty() {
        return String::new();
    }
    let preferences = memories
        .iter()
        .filter(|entry| entry.kind == MemoryKind::Preference)
        .count();
    let lessons = memories
        .iter()
        .filter(|entry| entry.kind == MemoryKind::Lesson)
        .count();
    format!("（preference {preferences} · lesson {lessons}）")
}

pub fn mem_list(store: &Store, all: bool, json: bool, kind: Option<&str>) -> Result<()> {
    let kind = parse_kind(kind)?;
    let today = Date::today();
    let memories = store.memories()?;
    let shown: Vec<&MemoryEntry> = memories
        .iter()
        .filter(|entry| all || !entry.is_expired(today))
        .filter(|entry| kind.is_none_or(|kind| entry.kind == kind))
        .collect();

    if json {
        let entries: Vec<MemoryJson> = shown.iter().map(|entry| MemoryJson::from(*entry)).collect();
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    let stale = memories
        .iter()
        .filter(|entry| entry.is_stale(today) && !entry.is_expired(today))
        .count();
    let expired = memories
        .iter()
        .filter(|entry| entry.is_expired(today))
        .count();
    println!(
        "{} 条记忆{} · {} 条待验证 · {} 条已过期（不再自动注入）",
        memories.len(),
        kind_summary(&memories),
        stale,
        expired
    );
    if shown.is_empty() {
        println!("（空。用 `niuse mem add \"...\"` 或等待对话中的候选。）");
        return Ok(());
    }
    for entry in shown {
        println!("{}", entry_line(entry, today));
    }
    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct MemoryJson<'a> {
    id: &'a str,
    kind: &'a str,
    tags: &'a [String],
    created: String,
    confirmed: String,
    age_days: i64,
    expired: bool,
    source: Option<&'a str>,
    text: &'a str,
}

impl<'a> From<&'a MemoryEntry> for MemoryJson<'a> {
    fn from(entry: &'a MemoryEntry) -> Self {
        let today = Date::today();
        Self {
            id: &entry.id,
            kind: entry.kind.as_str(),
            tags: &entry.tags,
            created: entry.created.to_string(),
            confirmed: entry.confirmed.to_string(),
            age_days: entry.age_days(today),
            expired: entry.is_expired(today),
            source: entry.source.as_deref(),
            text: &entry.text,
        }
    }
}

fn parse_kind(kind: Option<&str>) -> Result<Option<MemoryKind>> {
    kind.map(|value| {
        MemoryKind::parse(value)
            .with_context(|| format!("unknown kind {value:?}; use preference or lesson"))
    })
    .transpose()
}

fn entry_line(entry: &MemoryEntry, today: Date) -> String {
    let age = entry.age_days(today);
    let age_label = match age {
        0 => "今天".to_string(),
        1..=30 => format!("{age} 天前"),
        _ if entry.is_expired(today) => format!("{age} 天前 · 已过期"),
        _ => format!("{age} 天前 · 待验证"),
    };
    let tags = if entry.tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", entry.tags.join(", "))
    };
    format!(
        "m-{}  {}{}  {}  {}",
        entry.id,
        entry.kind.as_str(),
        tags,
        age_label,
        entry.text
    )
}

/// Search every memory, including expired ones: recall is exactly where old
/// entries are still useful, even though they are no longer auto-injected.
pub fn mem_search(
    store: &Store,
    query: &str,
    fresh_only: bool,
    json: bool,
    kind: Option<&str>,
) -> Result<()> {
    let kind = parse_kind(kind)?;
    let today = Date::today();
    let query = query.trim();
    if query.is_empty() {
        bail!("usage: niuse mem search <query>");
    }

    let memories = store.memories()?;
    let mut scored: Vec<(f64, &MemoryEntry)> = memories
        .iter()
        .filter(|entry| !fresh_only || !entry.is_expired(today))
        .filter(|entry| kind.is_none_or(|kind| entry.kind == kind))
        .map(|entry| (search_score(query, &entry.text, &entry.tags), entry))
        .filter(|(score, _)| *score >= 0.2)
        .collect();
    scored.sort_by(|left, right| {
        right
            .0
            .partial_cmp(&left.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if json {
        let entries: Vec<MemoryJson> = scored
            .iter()
            .map(|(_, entry)| MemoryJson::from(*entry))
            .collect();
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    if scored.is_empty() {
        println!("没有匹配「{query}」的记忆。");
        return Ok(());
    }

    println!("{} 条匹配「{query}」", scored.len());
    for (_, entry) in scored.iter().take(10) {
        println!("{}", entry_line(entry, today));
    }
    Ok(())
}

pub fn mem_add(
    store: &Store,
    text: &str,
    kind: &str,
    tags: Vec<String>,
    force: bool,
) -> Result<()> {
    let kind = MemoryKind::parse(kind)
        .with_context(|| format!("unknown kind {kind:?}; use preference or lesson"))?;
    let text = one_line(text.trim().to_string());
    if text.is_empty() {
        bail!("memory text is empty");
    }
    if let Some(reason) = secret_reason(&text) {
        if !force {
            bail!("refusing to store: looks like a {reason}; pass --force to override");
        }
        eprintln!("warning: overriding secret scan ({reason})");
    }
    let existing = store.memories()?;
    let similar: Vec<&MemoryEntry> = existing
        .iter()
        .filter(|entry| conflicts(&entry.text, &entry.tags, &text, &tags))
        .collect();
    if !similar.is_empty() && !force {
        for entry in similar {
            eprintln!("similar to m-{}: {}", entry.id, entry.text);
        }
        bail!("a similar memory exists; pass --force to add anyway");
    }

    let entry = store.add_memory(MemoryEntry::new(kind, text, tags, Date::today(), None))?;
    println!("m-{}", entry.id);
    Ok(())
}

pub fn mem_show(store: &Store, id: &str) -> Result<()> {
    let id = normalize_id(id);
    let memories = store.memories()?;
    let Some(entry) = memories.iter().find(|entry| entry.id == id) else {
        bail!("no memory m-{id}");
    };
    println!("m-{}", entry.id);
    println!("kind        {}", entry.kind.as_str());
    println!(
        "tags        {}",
        if entry.tags.is_empty() {
            "-".into()
        } else {
            entry.tags.join(", ")
        }
    );
    println!("created     {}", entry.created);
    println!("confirmed   {}", entry.confirmed);
    println!("source      {}", entry.source.as_deref().unwrap_or("-"));
    println!("text        {}", entry.text);
    Ok(())
}

pub fn mem_edit(store: &Store, id: &str) -> Result<()> {
    let id = normalize_id(id);
    let memories = store.memories()?;
    let Some(entry) = memories.iter().find(|entry| entry.id == id) else {
        bail!("no memory m-{id}");
    };
    let Some(text) = edit_text(&entry.text) else {
        println!("no change");
        return Ok(());
    };
    if text == entry.text {
        println!("no change");
        return Ok(());
    }
    if let Some(reason) = secret_reason(&text) {
        bail!("refusing to store: looks like a {reason}");
    }
    let Some(updated) = store.update_memory(&id, |entry| {
        entry.text = text;
        entry.confirmed = Date::today();
    })?
    else {
        bail!("memory m-{id} disappeared while editing; nothing was written");
    };
    if let Err(error) = store.append_metric("edit", &updated.id) {
        eprintln!("warning: cannot record metric: {error:#}");
    }
    println!("updated m-{}", updated.id);
    Ok(())
}

/// Open `$EDITOR` on a Markdown body. Returns `None` when the editor fails or
/// leaves the file empty.
pub fn mem_forget(store: &Store, id: &str) -> Result<()> {
    let id = normalize_id(id);
    if store.remove_memory(&id)? {
        println!("forgot m-{id}");
    } else {
        bail!("no memory m-{id}");
    }
    Ok(())
}

pub fn mem_confirm(store: &Store, id: &str) -> Result<()> {
    let id = normalize_id(id);
    let updated = store.update_memory(&id, |entry| entry.confirmed = Date::today())?;
    match updated {
        Some(entry) => println!("m-{} confirmed {}", entry.id, entry.confirmed),
        None => bail!("no memory m-{id}"),
    }
    Ok(())
}

pub fn mem_prune(store: &Store, apply: bool) -> Result<()> {
    let today = Date::today();
    let memories = store.memories()?;
    let expired: Vec<&MemoryEntry> = memories
        .iter()
        .filter(|entry| entry.is_expired(today))
        .collect();
    if expired.is_empty() {
        println!("没有超过 90 天的记忆。");
        return Ok(());
    }

    println!("{} 条已过期（不再注入）：", expired.len());
    for entry in &expired {
        println!("{}", entry_line(entry, today));
    }
    if !apply {
        println!("dry run；加 --apply 才会删除。");
        return Ok(());
    }

    let backup = backup_memories(store)?;
    println!("backup: {}", backup.display());
    for entry in &expired {
        store.remove_memory(&entry.id)?;
    }
    if let Err(error) = store.append_metric("prune", &expired.len().to_string()) {
        eprintln!("warning: cannot record metric: {error:#}");
    }
    println!("已删除 {} 条。", expired.len());
    Ok(())
}

fn backup_memories(store: &Store) -> Result<PathBuf> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or(0);
    let directory = store.root().join("backups").join(format!("prune-{stamp}"));
    fs::create_dir_all(&directory)
        .with_context(|| format!("create backup directory {}", directory.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&directory, fs::Permissions::from_mode(0o700));
    }
    for kind in [MemoryKind::Preference, MemoryKind::Lesson] {
        let source = store.memory_dir().join(kind.file());
        if !source.exists() {
            continue;
        }
        let target = directory.join(kind.file());
        fs::copy(&source, &target).with_context(|| format!("back up {}", source.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&target, fs::Permissions::from_mode(0o600));
        }
    }
    Ok(directory)
}

pub fn mem_review(store: &Store) -> Result<()> {
    let today = Date::today();
    let mut stale: Vec<MemoryEntry> = store
        .memories()?
        .into_iter()
        .filter(|entry| entry.is_stale(today))
        .collect();
    stale.sort_by_key(|entry| entry.confirmed);
    if stale.is_empty() {
        println!("没有需要复核的记忆。");
        return Ok(());
    }
    let expired = stale.iter().filter(|entry| entry.is_expired(today)).count();
    println!(
        "{} 条待复核（{} 条已过期，已过期不再注入）",
        stale.len(),
        expired
    );

    for entry in stale {
        println!();
        let expired = if entry.is_expired(today) {
            " · 已过期"
        } else {
            ""
        };
        println!(
            "m-{} · {} · 最后确认 {}（{} 天前）{}",
            entry.id,
            entry.kind.as_str(),
            entry.confirmed,
            entry.age_days(today),
            expired
        );
        println!("  {}", entry.text);
        print!("[y] 仍然成立  [d] 删除  [s] 跳过  [q] 退出 > ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            break;
        }
        match line.trim().to_lowercase().as_str() {
            "y" | "yes" => {
                store.update_memory(&entry.id, |entry| entry.confirmed = today)?;
                if let Err(error) = store.append_metric("mem_review_accept", &entry.id) {
                    eprintln!("warning: cannot record metric: {error:#}");
                }
                println!("  已确认");
            }
            "d" | "delete" => {
                store.remove_memory(&entry.id)?;
                if let Err(error) = store.append_metric("mem_review_delete", &entry.id) {
                    eprintln!("warning: cannot record metric: {error:#}");
                }
                println!("  已删除");
            }
            "q" | "quit" => break,
            _ => println!("  跳过"),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_add_refuses_a_similar_entry_without_force() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        mem_add(
            &store,
            "Rust 仓库一律用 cargo nextest",
            "lesson",
            Vec::new(),
            false,
        )
        .unwrap();
        assert!(mem_add(
            &store,
            "Rust 仓库一律用 cargo nextest 跑测试",
            "lesson",
            Vec::new(),
            false,
        )
        .is_err());
        assert_eq!(store.memories().unwrap().len(), 1);
        mem_add(
            &store,
            "Rust 仓库一律用 cargo nextest 跑测试",
            "lesson",
            Vec::new(),
            true,
        )
        .unwrap();
        assert_eq!(store.memories().unwrap().len(), 2);
    }

    #[test]
    fn prune_removes_only_expired_entries() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        let fresh = MemoryEntry::new(
            MemoryKind::Preference,
            "fresh",
            Vec::new(),
            Date::parse("2026-10-20").unwrap(),
            None,
        );
        let expired = MemoryEntry {
            id: "dead".into(),
            kind: MemoryKind::Lesson,
            text: "old".into(),
            tags: Vec::new(),
            created: Date::parse("2026-01-01").unwrap(),
            confirmed: Date::parse("2026-01-01").unwrap(),
            source: None,
        };
        store.add_memory(fresh).unwrap();
        store.add_memory(expired).unwrap();

        mem_prune(&store, true).unwrap();
        let remaining = store.memories().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].text, "fresh");

        let backup = fs::read_dir(store.root().join("backups"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let lessons = fs::read_to_string(backup.join("lessons.md")).unwrap();
        assert!(lessons.contains("old"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&backup).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o700);
        }
    }

    #[test]
    fn kind_summary_counts_preferences_and_lessons() {
        let preference = MemoryEntry::new(
            MemoryKind::Preference,
            "short paragraphs",
            vec![],
            Date::today(),
            None,
        );
        let lesson = MemoryEntry::new(
            MemoryKind::Lesson,
            "use nextest",
            vec![],
            Date::today(),
            None,
        );
        assert_eq!(
            kind_summary(&[preference, lesson]),
            "（preference 1 · lesson 1）"
        );
        assert_eq!(kind_summary(&[]), "");
    }

    #[test]
    fn kind_filter_parses_and_rejects() {
        assert_eq!(
            parse_kind(Some("lesson")).unwrap(),
            Some(MemoryKind::Lesson)
        );
        assert_eq!(parse_kind(None).unwrap(), None);
        assert!(parse_kind(Some("other")).is_err());
    }

    #[test]
    fn memory_json_is_machine_readable() {
        let entry = MemoryEntry::new(
            MemoryKind::Preference,
            "prefer short paragraphs",
            vec!["style".into()],
            Date::parse("2026-10-20").unwrap(),
            Some("session-1".into()),
        );
        let json = serde_json::to_value(MemoryJson::from(&entry)).unwrap();
        assert_eq!(json["id"], entry.id);
        assert_eq!(json["kind"], "preference");
        assert_eq!(json["expired"], false);
        assert_eq!(json["source"], "session-1");
    }

    #[test]
    fn memory_ids_accept_common_prefixes() {
        assert_eq!(normalize_id("m-7f3a"), "7f3a");
        assert_eq!(normalize_id("l-7f3a"), "7f3a");
        assert_eq!(normalize_id("7f3a"), "7f3a");
    }
}
