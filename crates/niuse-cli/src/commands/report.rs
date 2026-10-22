//! `niuse report`: one read-only weekly review across metrics and stores.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use niuse_core::{Date, Store};
use serde::Serialize;

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct Report {
    pub window_days: u64,
    pub active_days: usize,
    pub turns: usize,
    pub errors: usize,
    pub proposals: usize,
    pub accepted: usize,
    pub acceptance_percent: Option<usize>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub cache_hit_percent: Option<usize>,
    pub tokens_per_turn: Option<u64>,
    pub goals_open: usize,
    pub loops_open: usize,
    pub loops_overdue: usize,
    pub plan_open: usize,
    pub plan_total: usize,
    pub dogfood_days: usize,
    pub dogfood_acceptance_percent: Option<usize>,
}

pub fn report(store: &Store, days: u64, dogfood: Option<PathBuf>, json: bool) -> Result<()> {
    let report = build(store, days, dogfood.as_deref())?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print!("{}", render(&report));
    }
    Ok(())
}

pub fn build(store: &Store, days: u64, dogfood: Option<&Path>) -> Result<Report> {
    let days = days.max(1);
    let today = Date::today();
    let since = today.plus_days(-(days as i64 - 1)).start_millis_local();
    let metrics = store.metrics()?;
    let mut active = BTreeSet::new();
    let mut report = Report {
        window_days: days,
        ..Report::default()
    };
    for line in metrics.lines() {
        let mut parts = line.splitn(3, '\t');
        let Some(at) = parts.next().and_then(|value| value.parse::<u64>().ok()) else {
            continue;
        };
        if at < since {
            continue;
        }
        let Some(kind) = parts.next() else {
            continue;
        };
        let detail = parts.next().unwrap_or_default();
        active.insert(Date::from_millis_local(at));
        match kind {
            "turn" => {
                report.turns += 1;
                if detail == "error" {
                    report.errors += 1;
                }
            }
            "propose" => report.proposals += 1,
            "accept" => report.accepted += 1,
            "tokens" => {
                if let Some((input, output, cached)) = token_detail(detail) {
                    report.input_tokens += input;
                    report.output_tokens += output;
                    report.cached_tokens += cached;
                }
            }
            _ => {}
        }
    }
    report.active_days = active.len();
    report.acceptance_percent = percent(report.accepted, report.proposals, 100);
    report.cache_hit_percent = percent(
        report.cached_tokens as usize,
        report.input_tokens as usize,
        100,
    );
    report.tokens_per_turn = (report.turns > 0)
        .then(|| (report.input_tokens + report.output_tokens) / report.turns as u64);

    report.goals_open = store.goals()?.iter().filter(|goal| !goal.done).count();
    let loops = store.loops()?;
    report.loops_open = loops.iter().filter(|item| !item.done).count();
    report.loops_overdue = loops.iter().filter(|item| item.is_overdue(today)).count();
    let plan = store.plan()?;
    report.plan_total = plan.len();
    report.plan_open = plan.iter().filter(|step| !step.done).count();

    if let Some(path) = dogfood.filter(|path| path.is_file()) {
        let (count, accepted, proposed) = dogfood_counts(path)?;
        report.dogfood_days = count;
        report.dogfood_acceptance_percent = percent(accepted, proposed, 100);
    }
    Ok(report)
}

fn render(report: &Report) -> String {
    let mut out = String::from("# niuse weekly report\n\n");
    out.push_str(&format!(
        "Window: last {} day(s) · {} active\n",
        report.window_days, report.active_days
    ));
    out.push_str(&format!(
        "Turns: {} ({} error(s))\n",
        report.turns, report.errors
    ));
    out.push_str(&format!(
        "Candidates: {} proposed · {} accepted · {}\n",
        report.proposals,
        report.accepted,
        report
            .acceptance_percent
            .map(|percent| format!("{percent}%"))
            .unwrap_or_else(|| "n/a".into())
    ));
    out.push_str(&format!(
        "Tokens: {} in · {} out · {} cached · {} cache hit · {}\n",
        crate::render::compact_tokens(report.input_tokens),
        crate::render::compact_tokens(report.output_tokens),
        crate::render::compact_tokens(report.cached_tokens),
        report
            .cache_hit_percent
            .map(|percent| format!("{percent}%"))
            .unwrap_or_else(|| "n/a".into()),
        report
            .tokens_per_turn
            .map(|tokens| format!("{} / turn", crate::render::compact_tokens(tokens)))
            .unwrap_or_else(|| "n/a / turn".into())
    ));
    out.push_str(&format!(
        "Open: {} goal(s) · {} promise(s) ({} overdue) · {} plan step(s) of {}\n",
        report.goals_open,
        report.loops_open,
        report.loops_overdue,
        report.plan_open,
        report.plan_total
    ));
    out.push_str(&format!(
        "Dogfood: {} day(s) · {}\n",
        report.dogfood_days,
        report
            .dogfood_acceptance_percent
            .map(|percent| format!("{percent}% acceptance"))
            .unwrap_or_else(|| "n/a".into())
    ));
    out
}

fn token_detail(detail: &str) -> Option<(u64, u64, u64)> {
    let mut parts = detail.split(',');
    Some((
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ))
}

fn percent(numerator: usize, denominator: usize, cap: usize) -> Option<usize> {
    (denominator > 0).then(|| (numerator * 100 / denominator).min(cap))
}

fn dogfood_counts(path: &Path) -> Result<(usize, usize, usize)> {
    let text = std::fs::read_to_string(path)?;
    let (mut count, mut accepted, mut proposed) = (0, 0, 0);
    let mut in_table = false;
    for line in text.lines() {
        if line.trim() == "| date | turns | proposed | accepted | note |" {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        if !line.trim_start().starts_with('|') {
            break;
        }
        if line.contains("---") {
            continue;
        }
        let cells: Vec<&str> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        if cells.len() != 5 || Date::parse(cells[0]).is_none() {
            continue;
        }
        count += 1;
        proposed += cells[2].parse::<usize>().unwrap_or(0);
        accepted += cells[3].parse::<usize>().unwrap_or(0);
    }
    Ok((count, accepted, proposed))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(input: &str) -> Date {
        Date::parse(input).unwrap()
    }

    #[test]
    fn report_combines_metrics_and_store_counts() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        let today = date("2026-10-22");
        let at = today.start_millis_local();
        let other = today.plus_days(-1).start_millis_local();
        std::fs::write(
            store.metrics_path(),
            format!(
                "{at}\tturn\tok\n{other}\tturn\terror\n{at}\tpropose\tx\n{at}\taccept\tm-1\n{at}\ttokens\t1000,200,500\n"
            ),
        )
        .unwrap();
        store
            .add_goal(niuse_core::Goal::new("ship", today))
            .unwrap();
        store
            .add_loop(niuse_core::LoopItem::new(
                "promise",
                Some(today.plus_days(-1)),
            ))
            .unwrap();
        store.add_step("step".into(), today).unwrap();

        let report = build(&store, 7, None).unwrap();
        assert_eq!(report.turns, 2);
        assert_eq!(report.errors, 1);
        assert_eq!(report.acceptance_percent, Some(100));
        assert_eq!(report.cache_hit_percent, Some(50));
        assert_eq!(report.tokens_per_turn, Some(600));
        assert_eq!(report.goals_open, 1);
        assert_eq!(report.loops_overdue, 1);
        assert_eq!(report.plan_open, 1);
    }

    #[test]
    fn dogfood_counts_only_table_rows() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("dogfood.md");
        std::fs::write(
            &path,
            "| date | turns | proposed | accepted | note |\n|---|---|---|---|---|\n| 2026-10-21 | 1 | 2 | 1 | x |\n| 2026-10-22 | 1 | 0 | 0 | y |\n",
        )
        .unwrap();
        assert_eq!(dogfood_counts(&path).unwrap(), (2, 1, 2));
    }
}
