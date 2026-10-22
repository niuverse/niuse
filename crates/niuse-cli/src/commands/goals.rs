//! Human-owned goals: add, list, and close.

use anyhow::{bail, Result};
use niuse_core::policy::secret_reason;
use niuse_core::{Date, Goal, Store};
use serde::Serialize;

pub const MAX_OPEN_GOALS: usize = 7;

#[derive(Debug, Serialize)]
struct GoalJson {
    id: String,
    text: String,
    created: String,
    done: bool,
}

pub fn goals_list(store: &Store, all: bool, json: bool) -> Result<()> {
    let goals = store.goals()?;
    let shown: Vec<&Goal> = goals.iter().filter(|goal| all || !goal.done).collect();
    if json {
        let items: Vec<GoalJson> = shown
            .iter()
            .map(|goal| GoalJson {
                id: format!("g-{}", goal.id),
                text: goal.text.clone(),
                created: goal.created.format(),
                done: goal.done,
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }
    if shown.is_empty() {
        println!("没有进行中的目标。");
        return Ok(());
    }
    for goal in shown {
        println!("{}", goal.render());
    }
    Ok(())
}

pub fn goals_add(store: &Store, text: &str) -> Result<()> {
    let goal = goals_add_one(store, text)?;
    println!("目标已记录 g-{}", goal.id);
    Ok(())
}

/// Non-printing variant for the TUI, where the host owns the screen.
pub fn goals_add_one(store: &Store, text: &str) -> Result<Goal> {
    let text = text.trim();
    if text.is_empty() {
        bail!("usage: goals add <text>");
    }
    if let Some(reason) = secret_reason(text) {
        bail!("refusing to store: looks like a {reason}");
    }
    let open = store.goals()?.iter().filter(|goal| !goal.done).count();
    if open >= MAX_OPEN_GOALS {
        bail!(
            "已有 {} 个进行中的目标；先完成一个，再添加新的",
            MAX_OPEN_GOALS
        );
    }
    store.add_goal(Goal::new(text, Date::today()))
}

pub fn goals_done(store: &Store, id: &str) -> Result<()> {
    let id = goals_close(store, id)?;
    println!("目标 g-{id} 已完成");
    Ok(())
}

/// Non-printing close; returns the normalized id.
pub fn goals_close(store: &Store, id: &str) -> Result<String> {
    let id = normalize_id(id);
    if store.close_goal(&id)? {
        Ok(id)
    } else {
        bail!("未找到目标 g-{id}")
    }
}

fn normalize_id(id: &str) -> String {
    id.trim()
        .trim_start_matches("g-")
        .trim_start_matches('g')
        .trim_start_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> (tempfile::TempDir, Store) {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        (directory, store)
    }

    #[test]
    fn add_list_and_close_round_trip() {
        let (_directory, store) = temp_store();
        goals_add(&store, "ship the personal agent").unwrap();
        let goals = store.goals().unwrap();
        assert_eq!(goals.len(), 1);
        assert!(!goals[0].done);
        goals_done(&store, &format!("g-{}", goals[0].id)).unwrap();
        assert!(store.goals().unwrap()[0].done);
    }

    #[test]
    fn open_goal_limit_is_enforced() {
        let (_directory, store) = temp_store();
        for index in 0..MAX_OPEN_GOALS {
            goals_add(&store, &format!("goal {index}")).unwrap();
        }
        let error = goals_add(&store, "one too many").unwrap_err();
        assert!(error.to_string().contains("先完成"));
    }

    #[test]
    fn secrets_are_refused() {
        let (_directory, store) = temp_store();
        let error = goals_add(&store, "key sk-abcdefghijklmnopqrstuvwx").unwrap_err();
        assert!(error.to_string().contains("refusing"));
        assert!(store.goals().unwrap().is_empty());
    }
}
