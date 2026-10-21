//! Store-only commands. None of these construct a provider, so they work
//! offline and without credentials.

use anyhow::{bail, Context, Result};
use niuse_core::model::one_line;
use niuse_core::policy::{search_score, secret_reason};
use niuse_core::prompt::compose;
use niuse_core::{Date, JournalDay, LoopItem, MemoryEntry, MemoryKind, Store};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use niucode_auth::AuthStore;
use niucode_provider::Preset;

pub fn mem_list(store: &Store, all: bool) -> Result<()> {
    let today = Date::today();
    let memories = store.memories()?;
    let shown: Vec<&MemoryEntry> = memories
        .iter()
        .filter(|entry| all || !entry.is_expired(today))
        .collect();

    let stale = memories
        .iter()
        .filter(|entry| entry.is_stale(today) && !entry.is_expired(today))
        .count();
    let expired = memories
        .iter()
        .filter(|entry| entry.is_expired(today))
        .count();
    println!(
        "{} 条记忆 · {} 条待验证 · {} 条已过期（不再自动注入）",
        memories.len(),
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
pub fn mem_search(store: &Store, query: &str, fresh_only: bool) -> Result<()> {
    let today = Date::today();
    let query = query.trim();
    if query.is_empty() {
        bail!("usage: niuse mem search <query>");
    }

    let memories = store.memories()?;
    let mut scored: Vec<(f64, &MemoryEntry)> = memories
        .iter()
        .filter(|entry| !fresh_only || !entry.is_expired(today))
        .map(|entry| (search_score(query, &entry.text, &entry.tags), entry))
        .filter(|(score, _)| *score >= 0.2)
        .collect();
    scored.sort_by(|left, right| {
        right
            .0
            .partial_cmp(&left.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

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
    let updated = store
        .update_memory(&id, |entry| {
            entry.text = text;
            entry.confirmed = Date::today();
        })?
        .expect("entry existed when editing started");
    if let Err(error) = store.append_metric("edit", &updated.id) {
        eprintln!("warning: cannot record metric: {error:#}");
    }
    println!("updated m-{}", updated.id);
    Ok(())
}

/// Open `$EDITOR` on a Markdown body. Returns `None` when the editor fails or
/// leaves the file empty.
pub(crate) fn edit_markdown(initial: &str) -> Option<String> {
    let path = std::env::temp_dir().join(format!("niuse-edit-{}.md", std::process::id()));
    fs::write(&path, initial).ok()?;
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = std::process::Command::new(&editor).arg(&path).status();
    let text = fs::read_to_string(&path).ok();
    let _ = fs::remove_file(&path);
    status.ok()?.success().then_some(())?;
    text.map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

/// One-line variant used by candidate and memory editing.
pub(crate) fn edit_text(initial: &str) -> Option<String> {
    edit_markdown(initial)
        .map(one_line)
        .filter(|text| !text.is_empty())
}

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

    for entry in &expired {
        store.remove_memory(&entry.id)?;
    }
    if let Err(error) = store.append_metric("prune", &expired.len().to_string()) {
        eprintln!("warning: cannot record metric: {error:#}");
    }
    println!("已删除 {} 条。", expired.len());
    Ok(())
}

pub fn mem_review(store: &Store) -> Result<()> {
    let today = Date::today();
    let stale: Vec<MemoryEntry> = store
        .memories()?
        .into_iter()
        .filter(|entry| entry.is_stale(today))
        .collect();
    if stale.is_empty() {
        println!("没有需要复核的记忆。");
        return Ok(());
    }

    for entry in stale {
        println!();
        println!(
            "m-{} · {} · 最后确认 {}（{} 天前）",
            entry.id,
            entry.kind.as_str(),
            entry.confirmed,
            entry.age_days(today)
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
                println!("  已确认");
            }
            "d" | "delete" => {
                store.remove_memory(&entry.id)?;
                println!("  已删除");
            }
            "q" | "quit" => break,
            _ => println!("  跳过"),
        }
    }
    Ok(())
}

pub fn loops_list(store: &Store, all: bool) -> Result<()> {
    let today = Date::today();
    let loops = store.loops()?;
    let open = loops.iter().filter(|item| !item.done).count();
    println!("{} 条承诺 · {} 条未完成", loops.len(), open);
    for item in &loops {
        if item.done && !all {
            continue;
        }
        let mark = if item.done { "x" } else { " " };
        let due = match item.due {
            Some(due) if item.is_overdue(today) => format!("（截止 {due}，已逾期）"),
            Some(due) => format!("（截止 {due}）"),
            None => String::new(),
        };
        println!("[{mark}] l-{} {}{due}", item.id, item.text);
    }
    Ok(())
}

pub fn loops_add(store: &Store, text: &str, due: Option<&str>) -> Result<()> {
    let text = one_line(text.trim().to_string());
    if text.is_empty() {
        bail!("loop text is empty");
    }
    if let Some(reason) = secret_reason(&text) {
        bail!("refusing to store: looks like a {reason}");
    }
    let due = match due {
        Some(value) => {
            Some(Date::parse(value).with_context(|| format!("invalid due date {value:?}"))?)
        }
        None => None,
    };
    let item = store.add_loop(LoopItem::new(text, due))?;
    if let Err(error) = store.append_metric("loop", &item.id) {
        eprintln!("warning: cannot record metric: {error:#}");
    }
    match item.due {
        Some(due) => println!("l-{} {} (due {due})", item.id, item.text),
        None => println!("l-{} {}", item.id, item.text),
    }
    Ok(())
}

pub fn loops_done(store: &Store, id: &str) -> Result<()> {
    let id = normalize_id(id);
    if store.close_loop(&id)? {
        println!("closed l-{id}");
    } else {
        bail!("no loop l-{id}");
    }
    Ok(())
}

pub fn journal_show(store: &Store, days: usize) -> Result<()> {
    let days = store.recent_journal(days)?;
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

#[derive(Debug)]
struct ThreadInfo {
    path: PathBuf,
    active: bool,
    events: usize,
    bytes: u64,
    age_secs: u64,
    modified: SystemTime,
}

pub fn threads(store: &Store, active: &Path) -> Result<()> {
    let entries = collect_threads(store, active)?;
    if entries.is_empty() {
        println!("还没有主线。第一次对话会自动创建。");
        return Ok(());
    }
    for info in entries {
        let marker = if info.active { "*" } else { " " };
        let relative = info.path.strip_prefix(store.root()).unwrap_or(&info.path);
        println!(
            "{marker} {:<8} {:>4} events  {:>8}  {}",
            human_age(info.age_secs),
            info.events,
            human_bytes(info.bytes),
            relative.display()
        );
    }
    Ok(())
}

fn collect_threads(store: &Store, active: &Path) -> Result<Vec<ThreadInfo>> {
    let mut entries = Vec::new();
    push_thread(&mut entries, &store.session_path(), active)?;
    let directory = store.threads_dir();
    if directory.is_dir() {
        for entry in
            fs::read_dir(&directory).with_context(|| format!("read {}", directory.display()))?
        {
            let path = entry?.path();
            if path.extension().and_then(|extension| extension.to_str()) == Some("jsonl") {
                push_thread(&mut entries, &path, active)?;
            }
        }
    }
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.modified));
    Ok(entries)
}

fn push_thread(entries: &mut Vec<ThreadInfo>, path: &Path, active: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
    let age_secs = SystemTime::now()
        .duration_since(modified)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    let events = niucode_core::load(path)
        .map(|loaded| loaded.events.len())
        .unwrap_or(0);
    entries.push(ThreadInfo {
        path: path.to_path_buf(),
        active: path == active,
        events,
        bytes: metadata.len(),
        age_secs,
        modified,
    });
    Ok(())
}

fn human_age(seconds: u64) -> String {
    match seconds {
        0..=59 => "just now".to_string(),
        60..=3_599 => format!("{}m ago", seconds / 60),
        3_600..=86_399 => format!("{}h ago", seconds / 3_600),
        _ => format!("{}d ago", seconds / 86_400),
    }
}

fn human_bytes(bytes: u64) -> String {
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
    let text = crate::export::render(&loaded.events, "niuse thread");
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

pub fn journal_search(store: &Store, query: &str) -> Result<()> {
    let query = query.trim();
    if query.is_empty() {
        bail!("usage: niuse journal --search <query>");
    }
    let days = store.recent_journal(usize::MAX)?;
    let hits = rank_journal(query, &days);
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

pub fn identity_edit(store: &Store) -> Result<()> {
    let current = store
        .identity()?
        .unwrap_or_else(|| "# 你是 niuse\n\n描述它是谁、它的边界和说话方式。\n".to_string());
    let Some(updated) = edit_markdown(&current) else {
        println!("no change");
        return Ok(());
    };
    store.write_identity(&updated)?;
    if let Err(error) = store.append_metric("identity_edit", "") {
        eprintln!("warning: cannot record metric: {error:#}");
    }
    println!("updated {}", store.identity_path().display());
    Ok(())
}

pub fn identity_show(store: &Store) -> Result<()> {
    match store.identity()? {
        Some(text) => {
            println!("{}", store.identity_path().display());
            println!();
            print!("{text}");
        }
        None => println!(
            "还没有 identity.md。运行 `niuse identity init` 创建：{}",
            store.identity_path().display()
        ),
    }
    Ok(())
}

pub fn identity_init(store: &Store, force: bool) -> Result<()> {
    let path = store.identity_path();
    if path.exists() && !force {
        bail!(
            "{} already exists; pass --force to overwrite",
            path.display()
        );
    }

    let answers = [
        ask("你叫它什么名字", "niuse")?,
        ask("它是你的什么（搭档 / 同事 / 朋友）", "工程搭档")?,
        ask("用三个词描述它的性格", "直接、克制、有判断")?,
        ask("它应该坚持的一条原则", "不知道就说不知道")?,
        ask("它绝对不能做的一件事", "奉承我、替我做决定")?,
        ask("它说话的方式", "先给结论，再给理由；不说客套话")?,
        ask("它应该主动提醒你什么", "未完成的承诺和明显的风险")?,
        ask("它怎么称呼你", "你")?,
    ];

    let name = &answers[0];
    let relationship = &answers[1];
    let style = &answers[5];
    let text = format!(
        "# 你是 {name}\n\n\
         你是一个{relationship}，不是一个通用助手。你称呼对方为「{user}」。\n\n\
         ## 性格\n{persona}\n\n\
         ## 坚持\n- {principle}\n\n\
         ## 边界\n- {boundary}\n\n\
         ## 说话方式\n{style}\n\n\
         ## 主动提醒\n{reminder}\n",
        user = answers[7],
        persona = answers[2],
        principle = answers[3],
        boundary = answers[4],
        style = style,
        reminder = answers[6],
    );
    store.write_identity(&text)?;
    println!("wrote {}", path.display());
    Ok(())
}

pub fn status(store: &Store, thread: &Path) -> Result<()> {
    let today = Date::today();
    let memories = store.memories()?;
    let loops = store.loops()?;
    let journal = store.recent_journal(3)?;
    let identity = store.identity()?;
    let user = store.user()?;
    let composition = compose(
        identity.as_deref(),
        user.as_deref(),
        &memories,
        &loops,
        &journal,
        today,
    );

    println!("home      {}", store.root().display());
    println!("thread    {}", thread.display());
    println!(
        "identity  {}",
        if identity.is_some() {
            "已设置"
        } else {
            "未设置"
        }
    );
    println!(
        "memory    {} 条（{} 条待验证，{} 条已过期）",
        memories.len(),
        memories
            .iter()
            .filter(|entry| entry.is_stale(today) && !entry.is_expired(today))
            .count(),
        memories
            .iter()
            .filter(|entry| entry.is_expired(today))
            .count()
    );
    println!(
        "loops     {} 条未完成",
        loops.iter().filter(|item| !item.done).count()
    );
    println!(
        "prompt    ~{} tokens（注入 {} 条记忆，{} 条被预算或过期挡掉）",
        composition.tokens, composition.memory_injected, composition.memory_dropped
    );
    Ok(())
}

fn ask(prompt: &str, default: &str) -> Result<String> {
    print!("{prompt} [{default}]: ");
    let _ = io::stdout().flush();
    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        return Ok(default.to_string());
    }
    let answer = line.trim();
    Ok(if answer.is_empty() {
        default.to_string()
    } else {
        answer.to_string()
    })
}

fn normalize_id(id: &str) -> String {
    id.trim()
        .trim_start_matches("m-")
        .trim_start_matches('l')
        .trim_start_matches('-')
        .to_string()
}

/// Local acceptance counters. One tab-separated line per event:
/// `unix_ms<TAB>kind<TAB>detail`. Detail never contains memory text.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Metric {
    at_ms: u64,
    kind: String,
    detail: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Summary {
    turns: usize,
    cancelled: usize,
    errors: usize,
    proposals: usize,
    accepted: usize,
    discarded: usize,
    edited: usize,
    suppressed: usize,
    blocked: usize,
    journals: usize,
    loops: usize,
    remembers: usize,
    review_accepted: usize,
    review_discarded: usize,
}

impl Summary {
    /// Accepted over proposed. Edited and replace/keep decisions count as
    /// accepted; suppressed candidates stay in the denominator.
    fn acceptance_percent(&self) -> Option<usize> {
        (self.proposals > 0).then(|| self.accepted * 100 / self.proposals)
    }
}

pub fn stats(store: &Store, days: u64, trend: bool) -> Result<()> {
    let metrics = parse_metrics(&store.metrics()?);
    if metrics.is_empty() {
        println!("还没有指标。下一次对话开始后会自动记录。");
        return Ok(());
    }

    let since = if days == 0 {
        0
    } else {
        now_ms().saturating_sub(days.saturating_mul(86_400_000))
    };
    let summary = summarize(&metrics, since);
    let window = if days == 0 {
        "all time".to_string()
    } else {
        format!("last {days} day(s)")
    };

    println!("{window}");
    println!(
        "turns        {} ({} cancelled, {} errors)",
        summary.turns, summary.cancelled, summary.errors
    );
    println!(
        "candidates   {} proposed · {} accepted · {} discarded · {} edited · {} suppressed · {} blocked",
        summary.proposals,
        summary.accepted,
        summary.discarded,
        summary.edited,
        summary.suppressed,
        summary.blocked
    );
    match summary.acceptance_percent() {
        Some(percent) => println!("acceptance   {percent}% (accepted / proposed)"),
        None => println!("acceptance   n/a (no candidates yet)"),
    }
    println!(
        "activity     {} journal writes · {} loops · {} manual memories · {} review accepted · {} review discarded",
        summary.journals,
        summary.loops,
        summary.remembers,
        summary.review_accepted,
        summary.review_discarded
    );

    if trend {
        let daily = daily_summary(&metrics, since);
        if !daily.is_empty() {
            println!();
            println!("by day");
            for (date, day) in daily.iter().rev().take(14).rev() {
                let acceptance = day
                    .acceptance_percent()
                    .map(|percent| format!("{percent}%"))
                    .unwrap_or_else(|| "-".to_string());
                println!(
                    "{date}  {:>3} turns  {:>2}/{:<2} accepted  {acceptance}",
                    day.turns, day.accepted, day.proposals
                );
            }
        }
    }
    Ok(())
}

#[derive(Debug, Default)]
struct DaySummary {
    turns: usize,
    proposals: usize,
    accepted: usize,
}

impl DaySummary {
    fn acceptance_percent(&self) -> Option<usize> {
        (self.proposals > 0).then(|| self.accepted * 100 / self.proposals)
    }
}

fn daily_summary(metrics: &[Metric], since_ms: u64) -> BTreeMap<Date, DaySummary> {
    let mut days: BTreeMap<Date, DaySummary> = BTreeMap::new();
    for metric in metrics.iter().filter(|metric| metric.at_ms >= since_ms) {
        let date = Date::from_days((metric.at_ms / 86_400_000) as i64);
        let day = days.entry(date).or_default();
        match metric.kind.as_str() {
            "turn" => day.turns += 1,
            "propose" => day.proposals += 1,
            "accept" => day.accepted += 1,
            _ => {}
        }
    }
    days
}

fn parse_metrics(text: &str) -> Vec<Metric> {
    text.lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, '\t');
            let at_ms = parts.next()?.parse().ok()?;
            let kind = parts.next()?.to_string();
            let detail = parts.next().unwrap_or_default().to_string();
            Some(Metric {
                at_ms,
                kind,
                detail,
            })
        })
        .collect()
}

fn summarize(metrics: &[Metric], since_ms: u64) -> Summary {
    let mut summary = Summary::default();
    for metric in metrics.iter().filter(|metric| metric.at_ms >= since_ms) {
        match metric.kind.as_str() {
            "turn" => {
                summary.turns += 1;
                match metric.detail.as_str() {
                    "cancelled" => summary.cancelled += 1,
                    "error" => summary.errors += 1,
                    _ => {}
                }
            }
            "propose" => summary.proposals += 1,
            "accept" => summary.accepted += 1,
            "discard" => summary.discarded += 1,
            "edit" => summary.edited += 1,
            "suppress" => summary.suppressed += 1,
            "blocked" => summary.blocked += 1,
            "journal" => summary.journals += 1,
            "loop" => summary.loops += 1,
            "remember" => summary.remembers += 1,
            "review_accept" => summary.review_accepted += 1,
            "review_discard" => summary.review_discarded += 1,
            _ => {}
        }
    }
    summary
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Level {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Default)]
struct Report {
    failures: usize,
    warnings: usize,
}

impl Report {
    fn record(&mut self, level: Level, label: &str, detail: impl Into<String>) {
        let tag = match level {
            Level::Ok => "ok  ",
            Level::Warn => "warn",
            Level::Fail => "FAIL",
        };
        println!("[{tag}] {label:<11} {}", detail.into());
        match level {
            Level::Ok => {}
            Level::Warn => self.warnings += 1,
            Level::Fail => self.failures += 1,
        }
    }
}

pub fn doctor(store: &Store, preset: &str) -> Result<()> {
    let mut report = Report::default();

    match fs::metadata(store.root()) {
        Ok(metadata) => {
            let mode = mode_of(&metadata);
            let writable = write_probe(store.root());
            let level = if writable { Level::Ok } else { Level::Fail };
            report.record(
                level,
                "home",
                format!(
                    "{} ({mode}, {})",
                    store.root().display(),
                    if writable { "writable" } else { "not writable" }
                ),
            );
        }
        Err(error) => report.record(
            Level::Fail,
            "home",
            format!("{}: {error}", store.root().display()),
        ),
    }

    match store.identity()? {
        Some(text) if !text.trim().is_empty() => {
            report.record(Level::Ok, "identity", format!("set ({} bytes)", text.len()))
        }
        _ => report.record(
            Level::Warn,
            "identity",
            "missing; run `niuse identity init`",
        ),
    }

    match store.user()? {
        Some(text) if !text.trim().is_empty() => {
            report.record(Level::Ok, "user", format!("set ({} bytes)", text.len()))
        }
        _ => report.record(
            Level::Warn,
            "user",
            "missing; run `niuse bootstrap` or write user.md",
        ),
    }

    let today = Date::today();
    let memories = store.memories()?;
    report.record(
        Level::Ok,
        "memory",
        format!(
            "{} entries ({} stale, {} expired)",
            memories.len(),
            memories
                .iter()
                .filter(|entry| entry.is_stale(today) && !entry.is_expired(today))
                .count(),
            memories
                .iter()
                .filter(|entry| entry.is_expired(today))
                .count()
        ),
    );

    let thread = store.session_path();
    if thread.exists() {
        match niucode_core::load(&thread) {
            Ok(loaded) => report.record(
                Level::Ok,
                "thread",
                format!("{} ({} events)", thread.display(), loaded.events.len()),
            ),
            Err(error) => report.record(Level::Warn, "thread", format!("unreadable: {error:#}")),
        }
    } else {
        report.record(Level::Ok, "thread", "new; will be created on first run");
    }

    let metrics = store.metrics()?;
    report.record(
        Level::Ok,
        "metrics",
        format!("{} records", metrics.lines().count()),
    );

    match crate::settings::load(store) {
        Ok(_) if store.config_path().exists() => report.record(
            Level::Ok,
            "config",
            format!("{} (valid)", store.config_path().display()),
        ),
        Ok(_) => report.record(Level::Ok, "config", "defaults (no config.toml)"),
        Err(error) => report.record(Level::Fail, "config", format!("{error:#}")),
    }

    match Preset::parse(preset) {
        Some(preset) => match AuthStore::new() {
            Ok(auth) => match auth.resolve(preset.as_str()) {
                Ok(Some(resolved)) => report.record(
                    Level::Ok,
                    "credential",
                    format!("{} via {}", preset.as_str(), resolved.source),
                ),
                Ok(None) => report.record(
                    Level::Fail,
                    "credential",
                    format!("missing; run `niucode auth set {}`", preset.as_str()),
                ),
                Err(error) => report.record(Level::Fail, "credential", format!("{error}")),
            },
            Err(error) => report.record(Level::Fail, "credential", format!("{error}")),
        },
        None => report.record(
            Level::Fail,
            "credential",
            format!("unknown provider {preset:?}"),
        ),
    }

    if niucode_tools::sandbox_available() {
        report.record(Level::Ok, "sandbox", "bwrap available");
    } else {
        report.record(
            Level::Warn,
            "sandbox",
            "bwrap missing; approvals fall back to per-action prompts",
        );
    }

    match pin_status() {
        Some(PinStatus::Match(rev)) => report.record(
            Level::Ok,
            "pin",
            format!("{rev} matches the local checkout"),
        ),
        Some(PinStatus::Drift { pinned, local }) => report.record(
            Level::Warn,
            "pin",
            format!("pinned {pinned}, local {local}; run scripts/sync-niucode.sh"),
        ),
        Some(PinStatus::Missing) => report.record(
            Level::Warn,
            "pin",
            "niucode.rev found, sibling checkout missing",
        ),
        None => report.record(Level::Ok, "pin", "no niucode.rev (installed binary)"),
    }

    println!(
        "\ndoctor: {} failure(s), {} warning(s)",
        report.failures, report.warnings
    );
    if report.failures > 0 {
        bail!("doctor found {} failure(s)", report.failures);
    }
    Ok(())
}

enum PinStatus {
    Match(String),
    Drift { pinned: String, local: String },
    Missing,
}

fn pin_status() -> Option<PinStatus> {
    let root = repo_root()?;
    let pinned = fs::read_to_string(root.join("niucode.rev")).ok()?;
    let pinned = pinned.trim().to_string();
    if pinned.is_empty() {
        return None;
    }
    let Some(parent) = root.parent() else {
        return Some(PinStatus::Missing);
    };
    let checkout = parent.join("niuverse").join("niucode");
    if !checkout.is_dir() {
        return Some(PinStatus::Missing);
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(&checkout)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return Some(PinStatus::Missing);
    }
    let local = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let pinned_short = pinned.chars().take(local.len()).collect::<String>();
    if pinned_short == local {
        Some(PinStatus::Match(pinned_short))
    } else {
        Some(PinStatus::Drift {
            pinned: pinned_short,
            local,
        })
    }
}

fn repo_root() -> Option<PathBuf> {
    let mut directory = std::env::current_dir().ok()?;
    loop {
        if directory.join("niucode.rev").is_file() {
            return Some(directory);
        }
        if !directory.pop() {
            return None;
        }
    }
}

fn write_probe(directory: &std::path::Path) -> bool {
    let probe = directory.join(".doctor-write-test");
    match fs::write(&probe, b"probe") {
        Ok(()) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

#[cfg(unix)]
fn mode_of(metadata: &fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    format!("{:04o}", metadata.permissions().mode() & 0o777)
}

#[cfg(not(unix))]
fn mode_of(_metadata: &fs::Metadata) -> String {
    "mode n/a".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_parse_and_summarize() {
        let text = "1000\tturn\tok\n1100\tpropose\trust\n1200\taccept\tm-1\n1300\tpropose\tgit\n1400\tdiscard\t\n1500\tblocked\tAPI key\n1600\tturn\tcancelled\n";
        let metrics = parse_metrics(text);
        assert_eq!(metrics.len(), 7);
        let summary = summarize(&metrics, 0);
        assert_eq!(summary.turns, 2);
        assert_eq!(summary.cancelled, 1);
        assert_eq!(summary.proposals, 2);
        assert_eq!(summary.accepted, 1);
        assert_eq!(summary.blocked, 1);
        assert_eq!(summary.acceptance_percent(), Some(50));
    }

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
    fn daily_summary_groups_by_utc_day() {
        let metrics = parse_metrics(
            "0\tturn\tok\n1000\tpropose\trust\n2000\taccept\tm-1\n86400000\tturn\tok\n",
        );
        let days = daily_summary(&metrics, 0);
        assert_eq!(days.len(), 2);
        let first = days.get(&Date::from_days(0)).unwrap();
        assert_eq!(first.turns, 1);
        assert_eq!(first.proposals, 1);
        assert_eq!(first.accepted, 1);
        assert_eq!(first.acceptance_percent(), Some(100));
        let second = days.get(&Date::from_days(1)).unwrap();
        assert_eq!(second.turns, 1);
        assert_eq!(second.acceptance_percent(), None);
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

    #[test]
    fn memory_ids_accept_common_prefixes() {
        assert_eq!(normalize_id("m-7f3a"), "7f3a");
        assert_eq!(normalize_id("l-7f3a"), "7f3a");
        assert_eq!(normalize_id("7f3a"), "7f3a");
    }

    #[test]
    fn manual_loop_persists_with_due_date() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        loops_add(&store, "check the pin", Some("2026-10-30")).unwrap();
        let loops = store.loops().unwrap();
        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].text, "check the pin");
        assert_eq!(loops[0].due, Date::parse("2026-10-30"));
    }

    #[test]
    fn manual_loop_rejects_a_bad_due_date() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        assert!(loops_add(&store, "check", Some("2026-13-01")).is_err());
        assert!(store.loops().unwrap().is_empty());
    }

    #[test]
    fn report_counts_levels() {
        let mut report = Report::default();
        report.record(Level::Ok, "a", "b");
        report.record(Level::Warn, "c", "d");
        report.record(Level::Fail, "e", "f");
        assert_eq!(report.failures, 1);
        assert_eq!(report.warnings, 1);
    }

    #[test]
    fn window_filters_old_metrics() {
        let metrics = parse_metrics("1000\tturn\tok\n9999999999999\tturn\tok\n");
        let summary = summarize(&metrics, 5000);
        assert_eq!(summary.turns, 1);
    }
}
