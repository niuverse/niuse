//! Store-only commands. None of these construct a provider, so they work
//! offline and without credentials.

use anyhow::{bail, Context, Result};
use niuse_core::model::one_line;
use niuse_core::policy::{search_score, secret_reason};
use niuse_core::prompt::compose;
use niuse_core::{Date, LoopItem, MemoryEntry, MemoryKind, Store};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
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

pub fn status(store: &Store) -> Result<()> {
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
    println!("thread    {}", store.session_path().display());
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

pub fn stats(store: &Store, days: u64) -> Result<()> {
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
    Ok(())
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
