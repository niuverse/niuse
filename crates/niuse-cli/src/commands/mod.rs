//! Store-only commands. None of these construct a provider, so they work
//! offline and without credentials.

mod doctor;
mod editor;
mod evidence;
mod goals;
mod journal;
mod loops;
mod mcp;
mod memory;
mod plan;
mod profile;
mod projects;
mod recall;
mod session;
mod verify;
pub use doctor::doctor;
pub(crate) use editor::{edit_markdown, edit_text};
pub use evidence::evidence;
pub use goals::{goals_add, goals_done, goals_list};
#[cfg(feature = "tui")]
pub use goals::{goals_add_one, goals_close};
pub use journal::{journal_add, journal_search, journal_show};
pub use loops::{loops_add, loops_done, loops_list};
pub use mcp::{init as mcp_init, InitOptions as McpInitOptions};
pub use memory::{
    mem_add, mem_confirm, mem_edit, mem_forget, mem_list, mem_prune, mem_review, mem_search,
    mem_show,
};
pub use plan::{plan_add, plan_done, plan_list};
pub use profile::{identity_edit, identity_init, identity_show, user_edit, user_show};
pub use projects::projects;
pub use recall::{recall, recall_text};
#[cfg(feature = "tui")]
pub(crate) use session::write_export;
pub use session::{export, sessions, thread_at, threads};
#[cfg(feature = "tui")]
pub use verify::verify_shell_with_step;
pub use verify::{verify_shell, verify_with_step};

use anyhow::Result;
use niuse_core::prompt::compose_with_plan;
use niuse_core::{Brief, Date, MemoryKind, Store};
use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

pub fn status(store: &Store, thread: &Path, cwd: &Path, json: bool) -> Result<()> {
    let today = Date::today();
    let memories = store.memories()?;
    let loops = store.loops()?;
    let goals = store.goals()?;
    let journal = store.recent_journal(3)?;
    let identity = store.identity()?;
    let user = store.user()?;
    let registry = crate::projects::default_root()
        .ok()
        .map(|root| crate::projects::Registry::discover(&root));
    let current = registry.as_ref().and_then(|registry| registry.current(cwd));
    let plan = store.plan()?;
    let composition = compose_with_plan(
        identity.as_deref(),
        user.as_deref(),
        &goals,
        &plan,
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
    let preferences = memories
        .iter()
        .filter(|entry| entry.kind == MemoryKind::Preference)
        .count();
    let lessons = memories
        .iter()
        .filter(|entry| entry.kind == MemoryKind::Lesson)
        .count();
    let open_loops = loops.iter().filter(|item| !item.done).count();
    let open_goals = goals.iter().filter(|goal| !goal.done).count();

    if json {
        let payload = serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "home": store.root().display().to_string(),
            "thread": thread.display().to_string(),
            "identity": identity.is_some(),
            "user": user.is_some(),
            "memory": {
                "total": memories.len(),
                "preferences": preferences,
                "lessons": lessons,
                "stale": stale,
                "expired": expired,
            },
            "loops": {
                "total": loops.len(),
                "open": open_loops,
            },
            "goals": {
                "total": goals.len(),
                "open": open_goals,
            },
            "prompt": {
                "tokens": composition.tokens,
                "injected": composition.memory_injected,
                "dropped": composition.memory_dropped,
            },
            "project": current.map(|project| serde_json::json!({
                "key": project.key,
                "name": project.name(),
                "cwd": project.cwd.display().to_string(),
                "sessions": project.sessions,
                "activity": project.activity(),
                "last_seen": project.last_seen,
                "missing": project.missing,
            })),
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    println!("version   {}", env!("CARGO_PKG_VERSION"));
    println!("home      {}", store.root().display());
    println!("thread    {}", thread.display());
    match current {
        Some(project) => println!(
            "project   {} · {} 个 session · {}",
            project.name(),
            project.sessions,
            crate::projects::day_label(project.activity())
        ),
        None => println!("project   未登记"),
    }
    println!(
        "identity  {}",
        if identity.is_some() {
            "已设置"
        } else {
            "未设置"
        }
    );
    println!(
        "user      {}",
        if user.is_some() {
            "已设置"
        } else {
            "未设置"
        }
    );
    println!(
        "memory    {} 条（preference {} · lesson {}；{} 条待验证，{} 条已过期）",
        memories.len(),
        preferences,
        lessons,
        stale,
        expired
    );
    println!("loops     {} 条未完成", open_loops);
    println!("goals     {} 条进行中", open_goals);
    println!(
        "prompt    ~{} tokens（注入 {} 条记忆，{} 条被预算或过期挡掉）",
        composition.tokens, composition.memory_injected, composition.memory_dropped
    );
    Ok(())
}

/// Print what needs attention today. Store-only; no provider.
pub fn brief(store: &Store, cwd: &Path, json: bool) -> Result<()> {
    let view = Brief::build(store, Date::today(), project_name(cwd))?;
    if json {
        let payload = serde_json::json!({
            "today": view.today.to_string(),
            "project": view.project,
            "identity": view.has_identity,
            "user": view.has_user,
            "memory": {
                "total": view.memories,
                "stale": view.stale,
                "expired": view.expired,
            },
            "loops": {
                "open": view.open_loops,
                "overdue": view.overdue.iter().map(|item| serde_json::json!({
                    "id": item.id,
                    "text": item.text,
                    "due": item.due.map(|due| due.to_string()),
                })).collect::<Vec<_>>(),
            },
            "inbox": view.inbox,
            "journal": {
                "last": view.journal_last.map(|day| day.to_string()),
                "days_since": view.journal_last.map(|day| day.age_days(view.today)),
            },
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    print!("{}", view.render());
    Ok(())
}

/// Render the brief for the TUI, where the host owns the screen.
pub fn brief_text(store: &Store, cwd: &Path) -> Result<String> {
    Ok(Brief::build(store, Date::today(), project_name(cwd))?.render())
}

/// Only the actionable lines for the interactive startup banner.
pub fn brief_startup(store: &Store, cwd: &Path) -> Result<Vec<String>> {
    Ok(Brief::build(store, Date::today(), project_name(cwd))?.startup_lines())
}

fn project_name(cwd: &Path) -> String {
    cwd.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| cwd.display().to_string())
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
    mem_reviews_kept: usize,
    mem_reviews_edited: usize,
    mem_reviews_deleted: usize,
    input_tokens: u64,
    output_tokens: u64,
    cached_tokens: u64,
}

impl Summary {
    /// Accepted over proposed. Edited and replace/keep decisions count as
    /// accepted; suppressed candidates stay in the denominator.
    fn acceptance_percent(&self) -> Option<usize> {
        (self.proposals > 0).then(|| self.accepted * 100 / self.proposals)
    }

    fn cache_hit_percent(&self) -> Option<usize> {
        (self.input_tokens > 0)
            .then(|| (self.cached_tokens.saturating_mul(100) / self.input_tokens) as usize)
            .map(|percent| percent.min(100))
    }

    fn tokens_per_turn(&self) -> Option<u64> {
        (self.turns > 0).then(|| (self.input_tokens + self.output_tokens) / self.turns as u64)
    }
}

fn window_start(days: u64) -> u64 {
    if days == 0 {
        return 0;
    }
    let span = i64::try_from(days - 1).unwrap_or(i64::MAX);
    Date::today().plus_days(-span).start_millis_local()
}

pub fn stats(store: &Store, days: u64, trend: bool, json: bool) -> Result<()> {
    let metrics = parse_metrics(&store.metrics()?);
    let since = window_start(days);
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
        "tokens       {} in · {} out · {} cached",
        crate::render::compact_tokens(report.input_tokens),
        crate::render::compact_tokens(report.output_tokens),
        crate::render::compact_tokens(report.cached_tokens)
    );
    match report.cache_hit_percent {
        Some(percent) => println!("cache        {percent}% hit (cached / input)"),
        None => println!("cache        n/a (no input tokens yet)"),
    }
    match report.tokens_per_turn {
        Some(tokens) => println!(
            "per turn     {} tokens (input + output)",
            crate::render::compact_tokens(tokens)
        ),
        None => println!("per turn     n/a (no turns yet)"),
    }
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
        "activity     {} journal writes · {} loops · {} manual memories",
        report.journals, report.loops, report.remembers
    );
    println!(
        "reviews      {} bootstrap accepted · {} bootstrap discarded · {} memory kept · {} edited · {} deleted",
        report.review_accepted,
        report.review_discarded,
        report.mem_reviews_kept,
        report.mem_reviews_edited,
        report.mem_reviews_deleted
    );

    if trend && !report.days.is_empty() {
        println!();
        println!("by day");
        for day in report.days.iter().rev().take(14).rev() {
            let acceptance = day
                .acceptance_percent
                .map(|percent| format!("{percent}%"))
                .unwrap_or_else(|| "-".to_string());
            let cached = if day.cached_tokens > 0 {
                format!(
                    " · {} cached",
                    crate::render::compact_tokens(day.cached_tokens)
                )
            } else {
                String::new()
            };
            println!(
                "{}  {:>3} turns  {:>2}/{:<2} accepted  {acceptance}  {:>5} in · {:>4} out{cached}",
                day.date,
                day.turns,
                day.accepted,
                day.proposals,
                crate::render::compact_tokens(day.input_tokens),
                crate::render::compact_tokens(day.output_tokens)
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
    mem_reviews_kept: usize,
    mem_reviews_edited: usize,
    mem_reviews_deleted: usize,
    input_tokens: u64,
    output_tokens: u64,
    cached_tokens: u64,
    cache_hit_percent: Option<usize>,
    tokens_per_turn: Option<u64>,
    days: Vec<DayReport>,
}

#[derive(Debug, Serialize)]
struct DayReport {
    date: String,
    turns: usize,
    proposals: usize,
    accepted: usize,
    acceptance_percent: Option<usize>,
    input_tokens: u64,
    output_tokens: u64,
    cached_tokens: u64,
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
            mem_reviews_kept: summary.mem_reviews_kept,
            mem_reviews_edited: summary.mem_reviews_edited,
            mem_reviews_deleted: summary.mem_reviews_deleted,
            input_tokens: summary.input_tokens,
            output_tokens: summary.output_tokens,
            cached_tokens: summary.cached_tokens,
            cache_hit_percent: summary.cache_hit_percent(),
            tokens_per_turn: summary.tokens_per_turn(),
            days: daily
                .into_iter()
                .map(|(date, day)| DayReport {
                    date: date.to_string(),
                    turns: day.turns,
                    proposals: day.proposals,
                    accepted: day.accepted,
                    acceptance_percent: day.acceptance_percent(),
                    input_tokens: day.input_tokens,
                    output_tokens: day.output_tokens,
                    cached_tokens: day.cached_tokens,
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
    input_tokens: u64,
    output_tokens: u64,
    cached_tokens: u64,
}

impl DaySummary {
    fn acceptance_percent(&self) -> Option<usize> {
        (self.proposals > 0).then(|| self.accepted * 100 / self.proposals)
    }
}

fn parse_token_detail(detail: &str) -> Option<(u64, u64, u64)> {
    let mut parts = detail.split(',');
    let input = parts.next()?.parse().ok()?;
    let output = parts.next()?.parse().ok()?;
    let cached = parts.next()?.parse().ok()?;
    Some((input, output, cached))
}

fn daily_summary(metrics: &[Metric], since_ms: u64) -> BTreeMap<Date, DaySummary> {
    let mut days: BTreeMap<Date, DaySummary> = BTreeMap::new();
    for metric in metrics.iter().filter(|metric| metric.at_ms >= since_ms) {
        let date = Date::from_millis_local(metric.at_ms);
        let day = days.entry(date).or_default();
        match metric.kind.as_str() {
            "turn" => day.turns += 1,
            "propose" => day.proposals += 1,
            "accept" => day.accepted += 1,
            "tokens" => {
                if let Some((input, output, cached)) = parse_token_detail(&metric.detail) {
                    day.input_tokens += input;
                    day.output_tokens += output;
                    day.cached_tokens += cached;
                }
            }
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
            "mem_review_accept" => summary.mem_reviews_kept += 1,
            "mem_review_edit" => summary.mem_reviews_edited += 1,
            "mem_review_delete" => summary.mem_reviews_deleted += 1,
            "tokens" => {
                if let Some((input, output, cached)) = parse_token_detail(&metric.detail) {
                    summary.input_tokens += input;
                    summary.output_tokens += output;
                    summary.cached_tokens += cached;
                }
            }
            _ => {}
        }
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_parse_and_summarize() {
        let text = "1000\tturn\tok\n1100\tpropose\trust\n1200\taccept\tm-1\n1300\tpropose\tgit\n1400\tdiscard\t\n1500\tblocked\tAPI key\n1600\tturn\tcancelled\n1700\tmem_review_accept\tm-1\n1800\tmem_review_delete\tm-2\n1900\tmem_review_edit\tm-3\n2000\ttokens\t100,20,5\n2100\ttokens\t50,5,0\n";
        let metrics = parse_metrics(text);
        assert_eq!(metrics.len(), 12);
        let summary = summarize(&metrics, 0);
        assert_eq!(summary.turns, 2);
        assert_eq!(summary.cancelled, 1);
        assert_eq!(summary.proposals, 2);
        assert_eq!(summary.accepted, 1);
        assert_eq!(summary.blocked, 1);
        assert_eq!(summary.mem_reviews_kept, 1);
        assert_eq!(summary.mem_reviews_edited, 1);
        assert_eq!(summary.mem_reviews_deleted, 1);
        assert_eq!(summary.input_tokens, 150);
        assert_eq!(summary.output_tokens, 25);
        assert_eq!(summary.cached_tokens, 5);
        assert_eq!(summary.acceptance_percent(), Some(50));
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
    fn daily_summary_groups_by_local_day() {
        let metrics = parse_metrics(
            "0\tturn\tok\n1000\tpropose\trust\n2000\taccept\tm-1\n3000\ttokens\t100,20,5\n86400000\tturn\tok\n",
        );
        let days = daily_summary(&metrics, 0);
        assert_eq!(days.len(), 2);
        let first = days.get(&Date::from_millis_local(0)).unwrap();
        assert_eq!(first.turns, 1);
        assert_eq!(first.proposals, 1);
        assert_eq!(first.accepted, 1);
        assert_eq!(first.input_tokens, 100);
        assert_eq!(first.output_tokens, 20);
        assert_eq!(first.acceptance_percent(), Some(100));
        let second = days.get(&Date::from_millis_local(86_400_000)).unwrap();
        assert_eq!(second.turns, 1);
        assert_eq!(second.acceptance_percent(), None);
    }

    #[test]
    fn stats_window_starts_at_local_midnight() {
        assert_eq!(window_start(0), 0);
        assert_eq!(window_start(1), Date::today().start_millis_local());
        assert_eq!(
            window_start(2),
            Date::today().plus_days(-1).start_millis_local()
        );
    }

    #[test]
    fn window_filters_old_metrics() {
        let metrics = parse_metrics("1000\tturn\tok\n9999999999999\tturn\tok\n");
        let summary = summarize(&metrics, 5000);
        assert_eq!(summary.turns, 1);
    }
    #[test]
    fn cost_signals_are_derived_from_token_totals() {
        let summary = Summary {
            turns: 2,
            input_tokens: 1_000,
            output_tokens: 200,
            cached_tokens: 500,
            ..Summary::default()
        };
        assert_eq!(summary.cache_hit_percent(), Some(50));
        assert_eq!(summary.tokens_per_turn(), Some(600));
        assert_eq!(Summary::default().cache_hit_percent(), None);
        assert_eq!(Summary::default().tokens_per_turn(), None);
        let over = Summary {
            input_tokens: 10,
            cached_tokens: 20,
            ..Summary::default()
        };
        assert_eq!(over.cache_hit_percent(), Some(100));
    }
}
