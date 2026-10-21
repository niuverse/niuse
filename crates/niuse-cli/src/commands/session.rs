//! Session-facing commands: list threads and export a log as Markdown.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use niucode_core::Event;
use niuse_core::Store;

#[derive(Debug)]
struct ThreadInfo {
    path: PathBuf,
    active: bool,
    events: usize,
    bytes: u64,
    age_secs: u64,
    modified: SystemTime,
    title: Option<String>,
}

pub fn sessions(store: &Store, json: bool) -> Result<()> {
    let entries = collect_sessions(store)?;
    if json {
        let items: Vec<ThreadJson> = entries
            .iter()
            .map(|info| thread_json(store, info))
            .collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }
    if entries.is_empty() {
        println!("还没有一次性 session。`niuse exec` 会在这里留下日志。");
        return Ok(());
    }
    println!("{} 个一次性 session", entries.len());
    for info in entries {
        let relative = info.path.strip_prefix(store.root()).unwrap_or(&info.path);
        println!(
            "  {:<8} {:>4} events  {:>8}  {}",
            human_age(info.age_secs),
            info.events,
            human_bytes(info.bytes),
            relative.display()
        );
        if let Some(title) = &info.title {
            println!("      {title}");
        }
    }
    Ok(())
}

pub fn threads(store: &Store, active: &Path, json: bool) -> Result<()> {
    let entries = collect_threads(store, active)?;
    if json {
        let items: Vec<ThreadJson> = entries
            .iter()
            .map(|info| thread_json(store, info))
            .collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }
    if entries.is_empty() {
        println!("还没有主线。第一次对话会自动创建。");
        return Ok(());
    }
    for (index, info) in entries.into_iter().enumerate() {
        let marker = if info.active { "*" } else { " " };
        let relative = info.path.strip_prefix(store.root()).unwrap_or(&info.path);
        println!(
            "{marker} [{}] {:<8} {:>4} events  {:>8}  {}",
            index + 1,
            human_age(info.age_secs),
            info.events,
            human_bytes(info.bytes),
            relative.display()
        );
        if let Some(title) = &info.title {
            println!("    {title}");
        }
    }
    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct ThreadJson {
    path: String,
    active: bool,
    events: usize,
    bytes: u64,
    age_seconds: u64,
    title: Option<String>,
}

fn thread_json(store: &Store, info: &ThreadInfo) -> ThreadJson {
    ThreadJson {
        path: info
            .path
            .strip_prefix(store.root())
            .unwrap_or(&info.path)
            .to_string_lossy()
            .to_string(),
        active: info.active,
        events: info.events,
        bytes: info.bytes,
        age_seconds: info.age_secs,
        title: info.title.clone(),
    }
}

fn jsonl_files(directory: &Path) -> Result<Vec<PathBuf>> {
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(directory).with_context(|| format!("read {}", directory.display()))? {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("jsonl") {
            paths.push(path);
        }
    }
    Ok(paths)
}

/// One-based index into the same ordering `threads` prints.
pub fn thread_at(store: &Store, active: &Path, index: usize) -> Result<Option<PathBuf>> {
    if index == 0 {
        return Ok(None);
    }
    Ok(collect_threads(store, active)?
        .get(index - 1)
        .map(|info| info.path.clone()))
}

fn collect_threads(store: &Store, active: &Path) -> Result<Vec<ThreadInfo>> {
    let mut entries = Vec::new();
    push_thread(&mut entries, &store.session_path(), Some(active))?;
    for path in jsonl_files(&store.threads_dir())? {
        push_thread(&mut entries, &path, Some(active))?;
    }
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.modified));
    Ok(entries)
}

fn collect_sessions(store: &Store) -> Result<Vec<ThreadInfo>> {
    let mut entries = Vec::new();
    for path in jsonl_files(&store.root().join("sessions"))? {
        push_thread(&mut entries, &path, None)?;
    }
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.modified));
    Ok(entries)
}

fn push_thread(entries: &mut Vec<ThreadInfo>, path: &Path, active: Option<&Path>) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
    let age_secs = SystemTime::now()
        .duration_since(modified)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    let loaded = niucode_core::load(path);
    let events = loaded
        .as_ref()
        .map(|loaded| loaded.events.len())
        .unwrap_or(0);
    let title = loaded
        .as_ref()
        .ok()
        .and_then(|loaded| thread_title(&loaded.events));
    entries.push(ThreadInfo {
        path: path.to_path_buf(),
        active: active.is_some_and(|active| path == active),
        events,
        bytes: metadata.len(),
        age_secs,
        modified,
        title,
    });
    Ok(())
}

fn thread_title(events: &[Event]) -> Option<String> {
    for event in events {
        if let Event::UserMessage { text, .. } = event {
            let line = text.lines().next().unwrap_or(text).trim();
            if line.is_empty() {
                continue;
            }
            let title: String = line.chars().take(60).collect();
            return Some(if line.chars().count() > 60 {
                format!("{title}…")
            } else {
                title
            });
        }
    }
    None
}

fn human_age(seconds: u64) -> String {
    match seconds {
        0..=59 => "just now".to_string(),
        60..=3_599 => format!("{}m ago", seconds / 60),
        3_600..=86_399 => format!("{}h ago", seconds / 3_600),
        _ => format!("{}d ago", seconds / 86_400),
    }
}

pub(crate) fn human_bytes(bytes: u64) -> String {
    if bytes < 1_024 {
        format!("{bytes} B")
    } else if bytes < 1_048_576 {
        format!("{:.1} KiB", bytes as f64 / 1_024.0)
    } else {
        format!("{:.1} MiB", bytes as f64 / 1_048_576.0)
    }
}

pub fn export(store: &Store, out: Option<&Path>, session: Option<&Path>) -> Result<()> {
    let path = session
        .map(Path::to_path_buf)
        .unwrap_or_else(|| store.session_path());
    if !path.exists() {
        bail!("no session at {}", path.display());
    }
    let loaded = niucode_core::load(&path)?;
    for warning in &loaded.warnings {
        eprintln!("warning: {warning}");
    }
    let title = thread_title(&loaded.events).unwrap_or_else(|| "niuse thread".to_string());
    let text = crate::export::render(&loaded.events, &title);
    match out {
        Some(out) => {
            write_export(out, &text)?;
            println!("wrote {} ({} events)", out.display(), loaded.events.len());
        }
        None => print!("{text}"),
    }
    Ok(())
}

fn write_export(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create directory {}", parent.display()))?;
    }
    fs::write(path, text).with_context(|| format!("write {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("chmod 0600 {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_indexes_match_the_listing() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        fs::write(
            store.session_path(),
            "{\"type\":\"session_end\",\"ts\":0}\n",
        )
        .unwrap();
        fs::create_dir_all(store.threads_dir()).unwrap();
        fs::write(
            store.threads_dir().join("thread-1.jsonl"),
            "{\"type\":\"session_end\",\"ts\":0}\n",
        )
        .unwrap();
        let active = store.session_path();
        assert!(thread_at(&store, &active, 1).unwrap().is_some());
        assert!(thread_at(&store, &active, 2).unwrap().is_some());
        assert!(thread_at(&store, &active, 3).unwrap().is_none());
        assert!(thread_at(&store, &active, 0).unwrap().is_none());
    }

    #[test]
    fn sessions_listing_reads_one_shot_logs() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        let sessions = store.root().join("sessions");
        fs::create_dir_all(&sessions).unwrap();
        fs::write(
            sessions.join("run-1.jsonl"),
            "{\"type\":\"user_message\",\"text\":\"one shot\",\"ts\":1}\n",
        )
        .unwrap();
        let entries = collect_sessions(&store).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title.as_deref(), Some("one shot"));
        assert!(!entries[0].active);
    }

    #[test]
    fn thread_listing_reads_sessions() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        fs::write(
            store.session_path(),
            "{\"type\":\"session_end\",\"ts\":0}\n",
        )
        .unwrap();
        fs::create_dir_all(store.threads_dir()).unwrap();
        fs::write(
            store.threads_dir().join("thread-1.jsonl"),
            "{\"type\":\"session_end\",\"ts\":0}\n",
        )
        .unwrap();
        let entries = collect_threads(&store, &store.session_path()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries.iter().filter(|entry| entry.active).count(), 1);
        assert!(entries.iter().all(|entry| entry.events == 1));
    }

    #[test]
    fn thread_json_uses_relative_paths() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        let info = ThreadInfo {
            path: store.root().join("thread.jsonl"),
            active: true,
            events: 3,
            bytes: 120,
            age_secs: 5,
            modified: SystemTime::now(),
            title: None,
        };
        let json = serde_json::to_value(thread_json(&store, &info)).unwrap();
        assert_eq!(json["path"], "thread.jsonl");
        assert_eq!(json["active"], true);
        assert_eq!(json["events"], 3);
    }

    #[test]
    fn thread_titles_come_from_the_first_user_message() {
        let events: Vec<Event> = vec![
            serde_json::from_str(r#"{"type":"user_message","text":"hello there","ts":1}"#).unwrap(),
            serde_json::from_str(r#"{"type":"user_message","text":"second","ts":2}"#).unwrap(),
        ];
        assert_eq!(thread_title(&events), Some("hello there".to_string()));
        assert_eq!(thread_title(&[]), None);
    }

    #[test]
    fn human_readable_units() {
        assert_eq!(human_age(5), "just now");
        assert_eq!(human_age(120), "2m ago");
        assert_eq!(human_age(7_200), "2h ago");
        assert_eq!(human_age(172_800), "2d ago");
        assert_eq!(human_bytes(512), "512 B");
        assert_eq!(human_bytes(1_536), "1.5 KiB");
        assert_eq!(human_bytes(2 * 1_048_576), "2.0 MiB");
    }
}
