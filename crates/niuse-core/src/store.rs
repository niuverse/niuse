//! Filesystem layout for `~/.niuse`.
//!
//! Everything is Markdown. Mutations rewrite the affected file atomically and
//! keep it `0600`; a human can edit any of these files and the next load picks
//! the change up.

use std::cmp::Reverse;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::date::Date;
use crate::model::{
    parse_journal, parse_loop_line, parse_memory_section, JournalDay, LoopItem, MemoryEntry,
    MemoryKind,
};

const MEMORY_HEADER: &str = "<!-- 由 niuse 维护。可以手工编辑，但请保持每条的字段格式。 -->\n";

#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// `$NIUSE_HOME`, else `$HOME/.niuse`.
    pub fn discover() -> Result<Self> {
        if let Some(root) = std::env::var_os("NIUSE_HOME") {
            return Ok(Self::at(root));
        }
        let home = std::env::var_os("HOME").context("HOME is not set; set NIUSE_HOME instead")?;
        Ok(Self::at(PathBuf::from(home).join(".niuse")))
    }

    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn identity_path(&self) -> PathBuf {
        self.root.join("identity.md")
    }

    pub fn user_path(&self) -> PathBuf {
        self.root.join("user.md")
    }

    pub fn memory_dir(&self) -> PathBuf {
        self.root.join("memory")
    }

    pub fn journal_dir(&self) -> PathBuf {
        self.root.join("journal")
    }

    pub fn loops_path(&self) -> PathBuf {
        self.root.join("loops.md")
    }

    pub fn session_path(&self) -> PathBuf {
        self.root.join("thread.jsonl")
    }

    pub fn threads_dir(&self) -> PathBuf {
        self.root.join("threads")
    }

    pub fn history_path(&self) -> PathBuf {
        self.root.join("history")
    }

    pub fn config_path(&self) -> PathBuf {
        self.root.join("config.toml")
    }

    pub fn inbox_path(&self) -> PathBuf {
        self.root.join("inbox.md")
    }

    /// Create the directory skeleton. Missing prose files stay missing: an
    /// absent `identity.md` is a meaningful state, not an empty string.
    pub fn init(&self) -> Result<()> {
        create_private_dir(&self.root)?;
        create_private_dir(&self.memory_dir())?;
        create_private_dir(&self.journal_dir())?;
        Ok(())
    }

    pub fn identity(&self) -> Result<Option<String>> {
        read_optional(&self.identity_path())
    }

    pub fn user(&self) -> Result<Option<String>> {
        read_optional(&self.user_path())
    }

    pub fn write_identity(&self, text: &str) -> Result<()> {
        write_private(&self.identity_path(), text)
    }

    pub fn write_user(&self, text: &str) -> Result<()> {
        write_private(&self.user_path(), text)
    }

    pub fn memories(&self) -> Result<Vec<MemoryEntry>> {
        let mut entries = Vec::new();
        for kind in [MemoryKind::Preference, MemoryKind::Lesson] {
            let path = self.memory_dir().join(kind.file());
            let text = read_optional(&path)?.unwrap_or_default();
            entries.extend(parse_memory_file(&text, kind, Date::today()));
        }
        entries.sort_by_key(|entry| Reverse(entry.confirmed));
        Ok(entries)
    }

    /// Sections that look like memories but cannot be parsed. Used by doctor
    /// to catch a hand-edit that silently hid an entry.
    pub fn memory_issues(&self) -> Result<Vec<String>> {
        let entries = self.memories()?;
        let mut issues = Vec::new();
        for kind in [MemoryKind::Preference, MemoryKind::Lesson] {
            let path = self.memory_dir().join(kind.file());
            let text = read_optional(&path)?.unwrap_or_default();
            let headers = text
                .lines()
                .filter(|line| line.trim_start().starts_with("### m-"))
                .count();
            let parsed = entries.iter().filter(|entry| entry.kind == kind).count();
            if headers > parsed {
                issues.push(format!(
                    "{}: {} of {} section(s) could not be parsed",
                    kind.file(),
                    headers - parsed,
                    headers
                ));
            } else if parsed == 0 && has_stray_content(&text) {
                issues.push(format!(
                    "{}: has content but no parseable entries",
                    kind.file()
                ));
            }
        }
        Ok(issues)
    }

    /// Parse-integrity issues across memory, loops, and journal. Read-only;
    /// doctor uses it to surface content that a typo made invisible.
    pub fn content_issues(&self) -> Result<Vec<String>> {
        let mut issues = self.memory_issues()?;

        let loops_text = read_optional(&self.loops_path())?.unwrap_or_default();
        let loop_lines = loops_text
            .lines()
            .filter(|line| line.trim_start().starts_with("- ["))
            .count();
        let loops = self.loops()?;
        if loop_lines > loops.len() {
            issues.push(format!(
                "loops.md: {} of {} promise(s) could not be parsed",
                loop_lines - loops.len(),
                loop_lines
            ));
        }
        let stray_loops = loops_text
            .lines()
            .map(str::trim)
            .filter(|line| {
                !line.is_empty()
                    && !line.starts_with('#')
                    && !line.starts_with("<!--")
                    && !line.starts_with("-->")
                    && !line.starts_with("- [")
            })
            .count();
        if stray_loops > 0 {
            issues.push(format!("loops.md: {stray_loops} stray line(s)"));
        }

        if self.journal_dir().is_dir() {
            let mut files: Vec<PathBuf> = fs::read_dir(self.journal_dir())?
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
                .collect();
            files.sort();
            for path in files {
                let text = read_optional(&path)?.unwrap_or_default();
                let headings = text
                    .lines()
                    .filter(|line| line.trim_start().starts_with("## "))
                    .count();
                let bullets = text
                    .lines()
                    .filter(|line| line.trim_start().starts_with("- "))
                    .count();
                let parsed = parse_journal(&text);
                let parsed_bullets: usize = parsed.iter().map(|day| day.bullets.len()).sum();
                if headings > parsed.len() || bullets > parsed_bullets {
                    issues.push(format!(
                        "{}: {} heading(s), {} bullet(s); parsed {} day(s), {} bullet(s)",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        headings,
                        bullets,
                        parsed.len(),
                        parsed_bullets
                    ));
                }
                let stray = text
                    .lines()
                    .map(str::trim)
                    .filter(|line| {
                        !line.is_empty()
                            && !line.starts_with('#')
                            && !line.starts_with("<!--")
                            && !line.starts_with("-->")
                            && !line.starts_with("- ")
                    })
                    .count();
                if stray > 0 {
                    issues.push(format!(
                        "{}: {} stray line(s)",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        stray
                    ));
                }
            }
        }

        Ok(issues)
    }

    pub fn add_memory(&self, mut entry: MemoryEntry) -> Result<MemoryEntry> {
        let mut entries = self.memories()?;
        while entries.iter().any(|existing| existing.id == entry.id) {
            entry.id = crate::model::new_id(&format!("{}-{}", entry.text, entry.id));
        }
        entries.push(entry.clone());
        self.write_memories(&entries)?;
        Ok(entry)
    }

    pub fn update_memory(
        &self,
        id: &str,
        update: impl FnOnce(&mut MemoryEntry),
    ) -> Result<Option<MemoryEntry>> {
        let mut entries = self.memories()?;
        let Some(entry) = entries.iter_mut().find(|entry| entry.id == id) else {
            return Ok(None);
        };
        update(entry);
        let updated = entry.clone();
        self.write_memories(&entries)?;
        Ok(Some(updated))
    }

    pub fn remove_memory(&self, id: &str) -> Result<bool> {
        let mut entries = self.memories()?;
        let before = entries.len();
        entries.retain(|entry| entry.id != id);
        if entries.len() == before {
            return Ok(false);
        }
        self.write_memories(&entries)?;
        Ok(true)
    }

    pub fn write_memories(&self, entries: &[MemoryEntry]) -> Result<()> {
        for kind in [MemoryKind::Preference, MemoryKind::Lesson] {
            let mut text = format!("# {}\n\n{MEMORY_HEADER}\n", kind.title());
            for entry in entries.iter().filter(|entry| entry.kind == kind) {
                text.push_str(&entry.render());
                text.push('\n');
            }
            write_private(&self.memory_dir().join(kind.file()), &text)?;
        }
        Ok(())
    }

    pub fn loops(&self) -> Result<Vec<LoopItem>> {
        let text = read_optional(&self.loops_path())?.unwrap_or_default();
        Ok(parse_loops(&text))
    }

    pub fn write_loops(&self, loops: &[LoopItem]) -> Result<()> {
        let mut text = String::from("# 未完成的承诺\n\n<!-- 由 niuse 维护。--> \n\n");
        for item in loops {
            text.push_str(&item.render());
            text.push('\n');
        }
        write_private(&self.loops_path(), &text)
    }

    pub fn add_loop(&self, item: LoopItem) -> Result<LoopItem> {
        let mut loops = self.loops()?;
        loops.push(item.clone());
        self.write_loops(&loops)?;
        Ok(item)
    }

    pub fn close_loop(&self, id: &str) -> Result<bool> {
        let mut loops = self.loops()?;
        let Some(item) = loops.iter_mut().find(|item| item.id == id) else {
            return Ok(false);
        };
        item.done = true;
        self.write_loops(&loops)?;
        Ok(true)
    }

    /// Append bullets under the given day, creating the day section if needed.
    pub fn append_journal(&self, date: Date, bullets: &[String]) -> Result<bool> {
        let bullets: Vec<String> = bullets
            .iter()
            .map(|bullet| crate::model::one_line(bullet.trim_start_matches('-').to_string()))
            .filter(|bullet| !bullet.is_empty())
            .collect();
        if bullets.is_empty() {
            return Ok(false);
        }

        let path = self
            .journal_dir()
            .join(format!("{}.md", &date.format()[..7]));
        let mut days = parse_journal(&read_optional(&path)?.unwrap_or_default());
        match days.iter_mut().find(|day| day.date == date) {
            Some(day) => {
                for bullet in bullets {
                    if !day.bullets.contains(&bullet) {
                        day.bullets.push(bullet);
                    }
                }
            }
            None => days.push(JournalDay { date, bullets }),
        }
        days.sort_by_key(|day| day.date);

        let mut text = String::from(
            "# 经历\n\n<!-- 只记与上次不同的东西：决定、转折、未解问题、情绪变化。 -->\n\n",
        );
        for day in &days {
            text.push_str(&day.render());
            text.push('\n');
        }
        write_private(&path, &text)?;
        Ok(true)
    }

    pub fn read_inbox(&self) -> Result<Option<String>> {
        read_optional(&self.inbox_path())
    }

    pub fn write_inbox(&self, text: &str) -> Result<()> {
        write_private(&self.inbox_path(), text)
    }

    pub fn read_config(&self) -> Result<Option<String>> {
        read_optional(&self.config_path())
    }

    pub fn write_config(&self, text: &str) -> Result<()> {
        write_private(&self.config_path(), text)
    }

    pub fn metrics_path(&self) -> PathBuf {
        self.root.join("metrics.jsonl")
    }

    /// Append one tab-separated metric line.
    ///
    /// Metrics are local counters only: kinds, tags, and ids. Never memory text.
    pub fn append_metric(&self, kind: &str, detail: &str) -> Result<()> {
        let at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_millis())
            .unwrap_or(0);
        let line = format!(
            "{at}\t{}\t{}\n",
            sanitize_metric(kind),
            sanitize_metric(detail)
        );
        let path = self.metrics_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create directory {}", parent.display()))?;
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("open {}", path.display()))?;
        set_mode(&path, 0o600)?;
        file.write_all(line.as_bytes())
            .with_context(|| format!("append {}", path.display()))
    }

    pub fn metrics(&self) -> Result<String> {
        Ok(read_optional(&self.metrics_path())?.unwrap_or_default())
    }

    /// Newest first, at most `limit` days.
    pub fn recent_journal(&self, limit: usize) -> Result<Vec<JournalDay>> {
        let mut days = Vec::new();
        let directory = self.journal_dir();
        if !directory.is_dir() {
            return Ok(days);
        }
        for entry in fs::read_dir(&directory)
            .with_context(|| format!("read journal directory {}", directory.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            let text = fs::read_to_string(&path)
                .with_context(|| format!("read journal {}", path.display()))?;
            days.extend(parse_journal(&text));
        }
        days.sort_by_key(|day| Reverse(day.date));
        days.truncate(limit);
        Ok(days)
    }
}

fn has_stray_content(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim();
        !line.is_empty()
            && !line.starts_with('#')
            && !line.starts_with("<!--")
            && !line.starts_with("-->")
    })
}

fn parse_memory_file(text: &str, kind: MemoryKind, today: Date) -> Vec<MemoryEntry> {
    let mut entries = Vec::new();
    let mut section = String::new();
    for line in text.lines() {
        if line.trim_start().starts_with("### m-") {
            if let Some(entry) = parse_memory_section(section.trim(), kind, today) {
                entries.push(entry);
            }
            section.clear();
        }
        if !section.is_empty() || line.trim_start().starts_with("### m-") {
            section.push_str(line);
            section.push('\n');
        }
    }
    if let Some(entry) = parse_memory_section(section.trim(), kind, today) {
        entries.push(entry);
    }
    entries
}

fn parse_loops(text: &str) -> Vec<LoopItem> {
    text.lines().filter_map(parse_loop_line).collect()
}

fn sanitize_metric(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '\t' | '\n' | '\r' => ' ',
            other => other,
        })
        .collect()
}

fn read_optional(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("read {}", path.display())),
    }
}

fn create_private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create directory {}", path.display()))?;
    set_mode(path, 0o700)
}

fn write_private(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create directory {}", parent.display()))?;
    }
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, text).with_context(|| format!("write {}", temporary.display()))?;
    set_mode(&temporary, 0o600)?;
    fs::rename(&temporary, path).with_context(|| format!("replace {}", path.display()))?;
    Ok(())
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)
        .with_context(|| format!("stat {}", path.display()))?
        .permissions();
    permissions.set_mode(mode);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("chmod {:o} {}", mode, path.display()))
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (tempfile::TempDir, Store) {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        (directory, store)
    }

    fn date(input: &str) -> Date {
        Date::parse(input).unwrap()
    }

    #[test]
    fn memories_survive_add_and_reload() {
        let (_directory, store) = store();
        let entry = MemoryEntry::new(
            MemoryKind::Preference,
            "先给结论再展开",
            vec!["style".into()],
            date("2026-10-21"),
            Some("s1".into()),
        );
        let saved = store.add_memory(entry.clone()).unwrap();
        let loaded = store.memories().unwrap();
        assert_eq!(loaded, vec![saved]);
    }

    #[test]
    fn update_and_remove_work_by_id() {
        let (_directory, store) = store();
        let entry = store
            .add_memory(MemoryEntry::new(
                MemoryKind::Lesson,
                "用 nextest",
                vec![],
                date("2026-10-21"),
                None,
            ))
            .unwrap();

        let updated = store
            .update_memory(&entry.id, |entry| entry.confirmed = date("2026-10-23"))
            .unwrap()
            .expect("entry exists");
        assert_eq!(updated.confirmed, date("2026-10-23"));
        assert!(store.remove_memory(&entry.id).unwrap());
        assert!(store.memories().unwrap().is_empty());
    }

    #[test]
    fn journal_appends_to_one_day_and_sorts() {
        let (_directory, store) = store();
        store
            .append_journal(date("2026-10-20"), &["决定 A".into()])
            .unwrap();
        store
            .append_journal(date("2026-10-21"), &["转折 B".into()])
            .unwrap();
        store
            .append_journal(date("2026-10-21"), &["转折 B".into(), "未解 C".into()])
            .unwrap();

        let recent = store.recent_journal(3).unwrap();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].date, date("2026-10-21"));
        assert_eq!(recent[0].bullets, vec!["转折 B", "未解 C"]);
    }

    #[test]
    fn private_files_are_0600() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let (_directory, store) = store();
            store
                .add_memory(MemoryEntry::new(
                    MemoryKind::Preference,
                    "测试权限",
                    vec![],
                    date("2026-10-21"),
                    None,
                ))
                .unwrap();
            let path = store.memory_dir().join("preferences.md");
            let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn metrics_append_and_stay_private() {
        let (_directory, store) = store();
        store.append_metric("propose", "rust,testing").unwrap();
        store.append_metric("accept", "m-1234").unwrap();
        let text = store.metrics().unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].ends_with("\tpropose\trust,testing"));
        assert!(lines[1].ends_with("\taccept\tm-1234"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(store.metrics_path())
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn config_round_trips_privately() {
        let (_directory, store) = store();
        store.write_config("[defaults]\n").unwrap();
        assert_eq!(
            store.read_config().unwrap().as_deref(),
            Some("[defaults]\n")
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(store.config_path())
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn missing_identity_is_none() {
        let (_directory, store) = store();
        assert_eq!(store.identity().unwrap(), None);
    }

    #[test]
    fn loops_round_trip() {
        let (_directory, store) = store();
        let item = store
            .add_loop(LoopItem::new("查 cwd", Some(date("2026-10-22"))))
            .unwrap();
        let loaded = store.loops().unwrap();
        assert_eq!(loaded, vec![item.clone()]);
        assert!(store.close_loop(&item.id).unwrap());
        assert!(store.loops().unwrap()[0].done);
    }
}
