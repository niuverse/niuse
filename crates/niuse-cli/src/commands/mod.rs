//! Store-only commands. None of these construct a provider, so they work
//! offline and without credentials.

mod doctor;
mod editor;
mod loops;
mod memory;
pub use doctor::doctor;
pub(crate) use editor::{edit_markdown, edit_text};
pub use loops::{loops_add, loops_done, loops_list};
pub use memory::{
    mem_add, mem_confirm, mem_edit, mem_forget, mem_list, mem_prune, mem_review, mem_search,
    mem_show,
};

use anyhow::{bail, Context, Result};
use niuse_core::model::one_line;
use niuse_core::policy::{search_score, secret_reason};
use niuse_core::prompt::compose;
use niuse_core::{Date, JournalDay, Store};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

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

#[derive(Debug)]
struct ThreadInfo {
    path: PathBuf,
    active: bool,
    events: usize,
    bytes: u64,
    age_secs: u64,
    modified: SystemTime,
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

#[derive(Debug, serde::Serialize)]
struct ThreadJson {
    path: String,
    active: bool,
    events: usize,
    bytes: u64,
    age_seconds: u64,
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
    }
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

pub fn user_show(store: &Store) -> Result<()> {
    println!("{}", store.user_path().display());
    match store.user()? {
        Some(text) => print!("{text}"),
        None => println!("(missing; run `niuse bootstrap --profile` or `niuse user edit`)"),
    }
    Ok(())
}

pub fn user_edit(store: &Store) -> Result<()> {
    let current = store.user()?.unwrap_or_else(|| {
        "# 关于我\n\n- 我在做什么：\n- 我的工作方式：\n- 我的雷区：\n".to_string()
    });
    let Some(updated) = edit_markdown(&current) else {
        println!("no change");
        return Ok(());
    };
    if let Some(reason) = secret_reason(&updated) {
        bail!("refusing to save user.md: looks like a {reason}");
    }
    store.write_user(&updated)?;
    if let Err(error) = store.append_metric("user_edit", "") {
        eprintln!("warning: cannot record metric: {error:#}");
    }
    println!("updated {}", store.user_path().display());
    Ok(())
}

pub fn identity_edit(store: &Store) -> Result<()> {
    let current = store
        .identity()?
        .unwrap_or_else(|| "# 你是 niuse\n\n描述它是谁、它的边界和说话方式。\n".to_string());
    let Some(updated) = edit_markdown(&current) else {
        println!("no change");
        return Ok(());
    };
    if let Some(reason) = secret_reason(&updated) {
        bail!("refusing to save identity.md: looks like a {reason}");
    }
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
    if let Some(reason) = secret_reason(&text) {
        bail!("refusing to save identity.md: looks like a {reason}");
    }
    store.write_identity(&text)?;
    println!("wrote {}", path.display());
    Ok(())
}

pub fn status(store: &Store, thread: &Path, json: bool) -> Result<()> {
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

    let stale = memories
        .iter()
        .filter(|entry| entry.is_stale(today) && !entry.is_expired(today))
        .count();
    let expired = memories
        .iter()
        .filter(|entry| entry.is_expired(today))
        .count();
    let open_loops = loops.iter().filter(|item| !item.done).count();

    if json {
        let payload = serde_json::json!({
            "home": store.root().display().to_string(),
            "thread": thread.display().to_string(),
            "identity": identity.is_some(),
            "user": user.is_some(),
            "memory": {
                "total": memories.len(),
                "stale": stale,
                "expired": expired,
            },
            "loops": {
                "total": loops.len(),
                "open": open_loops,
            },
            "prompt": {
                "tokens": composition.tokens,
                "injected": composition.memory_injected,
                "dropped": composition.memory_dropped,
            },
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

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
        stale,
        expired
    );
    println!("loops     {} 条未完成", open_loops);
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

pub fn stats(store: &Store, days: u64, trend: bool, json: bool) -> Result<()> {
    let metrics = parse_metrics(&store.metrics()?);
    let since = if days == 0 {
        0
    } else {
        now_ms().saturating_sub(days.saturating_mul(86_400_000))
    };
    let summary = summarize(&metrics, since);
    let report = StatsReport::from_summary(&summary, days, daily_summary(&metrics, since));

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }
    if metrics.is_empty() {
        println!("还没有指标。下一次对话开始后会自动记录。");
        return Ok(());
    }

    let window = if days == 0 {
        "all time".to_string()
    } else {
        format!("last {days} day(s)")
    };
    println!("{window}");
    println!(
        "turns        {} ({} cancelled, {} errors)",
        report.turns, report.cancelled, report.errors
    );
    println!(
        "candidates   {} proposed · {} accepted · {} discarded · {} edited · {} suppressed · {} blocked",
        report.proposals,
        report.accepted,
        report.discarded,
        report.edited,
        report.suppressed,
        report.blocked
    );
    match report.acceptance_percent {
        Some(percent) => println!("acceptance   {percent}% (accepted / proposed)"),
        None => println!("acceptance   n/a (no candidates yet)"),
    }
    println!(
        "activity     {} journal writes · {} loops · {} manual memories · {} review accepted · {} review discarded",
        report.journals,
        report.loops,
        report.remembers,
        report.review_accepted,
        report.review_discarded
    );

    if trend && !report.days.is_empty() {
        println!();
        println!("by day");
        for day in report.days.iter().rev().take(14).rev() {
            let acceptance = day
                .acceptance_percent
                .map(|percent| format!("{percent}%"))
                .unwrap_or_else(|| "-".to_string());
            println!(
                "{}  {:>3} turns  {:>2}/{:<2} accepted  {acceptance}",
                day.date, day.turns, day.accepted, day.proposals
            );
        }
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct StatsReport {
    window_days: u64,
    turns: usize,
    cancelled: usize,
    errors: usize,
    proposals: usize,
    accepted: usize,
    discarded: usize,
    edited: usize,
    suppressed: usize,
    blocked: usize,
    acceptance_percent: Option<usize>,
    journals: usize,
    loops: usize,
    remembers: usize,
    review_accepted: usize,
    review_discarded: usize,
    days: Vec<DayReport>,
}

#[derive(Debug, Serialize)]
struct DayReport {
    date: String,
    turns: usize,
    proposals: usize,
    accepted: usize,
    acceptance_percent: Option<usize>,
}

impl StatsReport {
    fn from_summary(summary: &Summary, days: u64, daily: BTreeMap<Date, DaySummary>) -> Self {
        Self {
            window_days: days,
            turns: summary.turns,
            cancelled: summary.cancelled,
            errors: summary.errors,
            proposals: summary.proposals,
            accepted: summary.accepted,
            discarded: summary.discarded,
            edited: summary.edited,
            suppressed: summary.suppressed,
            blocked: summary.blocked,
            acceptance_percent: summary.acceptance_percent(),
            journals: summary.journals,
            loops: summary.loops,
            remembers: summary.remembers,
            review_accepted: summary.review_accepted,
            review_discarded: summary.review_discarded,
            days: daily
                .into_iter()
                .map(|(date, day)| DayReport {
                    date: date.to_string(),
                    turns: day.turns,
                    proposals: day.proposals,
                    accepted: day.accepted,
                    acceptance_percent: day.acceptance_percent(),
                })
                .collect(),
        }
    }
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
    fn stats_report_serializes_for_dogfood() {
        let metrics = parse_metrics("0\tturn\tok\n1000\tpropose\trust\n2000\taccept\tm-1\n");
        let summary = summarize(&metrics, 0);
        let report = StatsReport::from_summary(&summary, 0, daily_summary(&metrics, 0));
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"acceptance_percent\":100"));
        assert!(json.contains("\"date\":\"1970-01-01\""));
        assert!(json.contains("\"window_days\":0"));
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
        };
        let json = serde_json::to_value(thread_json(&store, &info)).unwrap();
        assert_eq!(json["path"], "thread.jsonl");
        assert_eq!(json["active"], true);
        assert_eq!(json["events"], 3);
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
    fn window_filters_old_metrics() {
        let metrics = parse_metrics("1000\tturn\tok\n9999999999999\tturn\tok\n");
        let summary = summarize(&metrics, 5000);
        assert_eq!(summary.turns, 1);
    }
}
