//! `niuse dogfood`: append or replace one day's row from local metrics.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use niuse_core::policy::secret_reason;
use niuse_core::{Date, Store};
use serde::Serialize;

const HEADER: &str = "| date | turns | proposed | accepted | note |";
const SEPARATOR: &str = "|---|---|---|---|---|";

#[derive(Debug, Serialize)]
struct Row {
    date: String,
    turns: usize,
    proposed: usize,
    accepted: usize,
    note: String,
    file: String,
}

pub fn dogfood(
    store: &Store,
    file: Option<PathBuf>,
    note: Option<String>,
    json: bool,
) -> Result<()> {
    let today = Date::today();
    let note = note.unwrap_or_else(|| "-".to_string());
    let note = note.trim().replace('|', "\\|");
    if let Some(reason) = secret_reason(&note) {
        bail!("refusing to write what looks like a secret: {reason}");
    }
    let (turns, proposed, accepted) = counts(&store.metrics()?, today);
    let path = file.unwrap_or_else(|| store.root().join("dogfood.md"));
    let text = if path.exists() {
        fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?
    } else {
        format!("{HEADER}\n{SEPARATOR}\n")
    };
    let updated = upsert(&text, today, turns, proposed, accepted, &note)?;
    if path.exists() {
        fs::write(&path, &updated).with_context(|| format!("write {}", path.display()))?;
    } else {
        write_private(&path, &updated)?;
    }
    let row = Row {
        date: today.to_string(),
        turns,
        proposed,
        accepted,
        note,
        file: path.display().to_string(),
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&row)?);
    } else {
        println!(
            "{} · {} turns · {} proposed · {} accepted · {}",
            row.date,
            row.turns,
            row.proposed,
            row.accepted,
            if row.note.is_empty() { "-" } else { &row.note }
        );
        println!("wrote {}", path.display());
    }
    Ok(())
}

fn counts(metrics: &str, today: Date) -> (usize, usize, usize) {
    let (mut turns, mut proposed, mut accepted) = (0, 0, 0);
    for line in metrics.lines() {
        let mut parts = line.splitn(3, '\t');
        let Some(at) = parts.next().and_then(|value| value.parse::<u64>().ok()) else {
            continue;
        };
        if Date::from_millis_local(at) != today {
            continue;
        }
        match parts.next() {
            Some("turn") => turns += 1,
            Some("propose") => proposed += 1,
            Some("accept") => accepted += 1,
            _ => {}
        }
    }
    (turns, proposed, accepted)
}

fn upsert(
    text: &str,
    today: Date,
    turns: usize,
    proposed: usize,
    accepted: usize,
    note: &str,
) -> Result<String> {
    let mut lines: Vec<String> = text.lines().map(ToString::to_string).collect();
    let Some(header) = lines.iter().position(|line| line.trim() == HEADER) else {
        bail!("no dogfood table header; expected `{HEADER}`");
    };
    if lines.get(header + 1).map(|line| line.trim()) != Some(SEPARATOR) {
        bail!("dogfood table is missing its separator row");
    }
    let row = format!("| {today} | {turns} | {proposed} | {accepted} | {note} |");
    let date_prefix = format!("| {today} |");
    let mut end = header + 2;
    while end < lines.len() && lines[end].trim_start().starts_with('|') {
        end += 1;
    }
    if let Some(existing) = lines[header + 2..end]
        .iter()
        .position(|line| line.trim_start().starts_with(&date_prefix))
    {
        lines[header + 2 + existing] = row;
    } else {
        lines.insert(end, row);
    }
    let mut out = lines.join("\n");
    out.push('\n');
    Ok(out)
}

fn write_private(path: &Path, body: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, body).with_context(|| format!("write {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("chmod {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(input: &str) -> Date {
        Date::parse(input).unwrap()
    }

    #[test]
    fn counts_only_today_turns_proposals_and_accepts() {
        let today = date("2026-10-22");
        let today_ms = today.start_millis_local();
        let yesterday_ms = today.plus_days(-1).start_millis_local();
        let metrics = format!(
            "{today_ms}\tturn\tok\n{today_ms}\tpropose\tx\n{today_ms}\taccept\tm-1\n{yesterday_ms}\tturn\tok\n{today_ms}\tjournal\tx\n"
        );
        assert_eq!(counts(&metrics, today), (1, 1, 1));
        assert_eq!(counts("garbage\n", today), (0, 0, 0));
    }

    #[test]
    fn upsert_replaces_the_same_day_and_keeps_other_rows() {
        let text = format!(
            "{HEADER}\n{SEPARATOR}\n| 2026-10-21 | 1 | 0 | 0 | old |\n| 2026-10-22 | 2 | 1 | 1 | first |\n"
        );
        let updated = upsert(&text, date("2026-10-22"), 3, 2, 1, "second").unwrap();
        assert_eq!(updated.matches("| 2026-10-22 |").count(), 1);
        assert!(updated.contains("| 2026-10-22 | 3 | 2 | 1 | second |"));
        assert!(updated.contains("| 2026-10-21 | 1 | 0 | 0 | old |"));
    }

    #[test]
    fn upsert_appends_a_new_day_after_the_last_row() {
        let text = format!("{HEADER}\n{SEPARATOR}\n| 2026-10-21 | 1 | 0 | 0 | old |\n");
        let updated = upsert(&text, date("2026-10-22"), 1, 0, 0, "-").unwrap();
        let lines: Vec<&str> = updated.lines().collect();
        assert_eq!(lines[3], "| 2026-10-22 | 1 | 0 | 0 | - |");
    }

    #[test]
    fn a_missing_table_is_an_error_not_a_silent_append() {
        assert!(upsert("no table\n", date("2026-10-22"), 1, 0, 0, "-").is_err());
    }
}
