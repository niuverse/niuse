//! Open-loop commands: list, add, and close promises.

use anyhow::{bail, Context, Result};
use niuse_core::model::one_line;
use niuse_core::policy::secret_reason;
use niuse_core::{Date, LoopItem, Store};
use serde::Serialize;

use super::normalize_id;

#[derive(Debug, Serialize)]
struct LoopJson {
    id: String,
    text: String,
    due: Option<String>,
    done: bool,
    overdue: bool,
}

pub fn loops_list(store: &Store, all: bool, json: bool) -> Result<()> {
    let today = Date::today();
    let loops = store.loops()?;
    if json {
        let items: Vec<LoopJson> = loops
            .iter()
            .filter(|item| all || !item.done)
            .map(|item| LoopJson {
                id: format!("l-{}", item.id),
                text: item.text.clone(),
                due: item.due.map(|due| due.to_string()),
                done: item.done,
                overdue: item.is_overdue(today),
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
