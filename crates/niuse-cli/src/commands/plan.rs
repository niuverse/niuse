//! Human-owned plan steps and the verification gate for closing one.

use std::path::Path;

use anyhow::{bail, Result};
use niucode_core::{load, Event};
use niuse_core::policy::secret_reason;
use niuse_core::{Date, PlanStep, Store};
use serde::Serialize;

pub const MAX_OPEN_STEPS: usize = 12;

#[derive(Debug, Serialize)]
struct StepJson {
    id: String,
    text: String,
    created: String,
    done: bool,
}

fn json(step: &PlanStep) -> StepJson {
    StepJson {
        id: step.label(),
        text: step.text.clone(),
        created: step.created.format(),
        done: step.done,
    }
}

pub fn plan_list(store: &Store, all: bool, json_output: bool) -> Result<()> {
    let steps = store.plan()?;
    let shown: Vec<&PlanStep> = steps.iter().filter(|step| all || !step.done).collect();
    if json_output {
        let items: Vec<StepJson> = shown.iter().map(|step| json(step)).collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }
    if shown.is_empty() {
        println!("没有进行中的步骤。");
        return Ok(());
    }
    for step in shown {
        let mark = if step.done { "x" } else { " " };
        println!("[{mark}] {} {}", step.label(), step.text);
    }
    Ok(())
}

pub fn plan_add(store: &Store, text: &str) -> Result<PlanStep> {
    let text = text.trim();
    if text.is_empty() {
        bail!("plan add needs step text");
    }
    if let Some(reason) = secret_reason(text) {
        bail!("refusing to store what looks like a secret: {reason}");
    }
    let open = store.plan()?.iter().filter(|step| !step.done).count();
    if open >= MAX_OPEN_STEPS {
        bail!("at most {MAX_OPEN_STEPS} open steps; close or remove one first");
    }
    store.add_step(text.to_string(), Date::today())
}

pub fn plan_done(store: &Store, thread: &Path, id: u32) -> Result<PlanStep> {
    let label = format!("p-{id}");
    let step = store
        .plan()?
        .into_iter()
        .find(|step| step.id == id)
        .ok_or_else(|| anyhow::anyhow!("unknown plan step {label}"))?;
    if step.done {
        bail!("{label} is already done");
    }
    let loaded = load(thread)?;
    let verified = loaded.events.iter().any(|event| {
        matches!(
            event,
            Event::Verification {
                step: Some(step),
                ok: true,
                ..
            } if step == &label
        )
    });
    if !verified {
        bail!(
            "{label} has no passing verification; run `niuse verify --step {label} -- <command>` first"
        );
    }
    store
        .close_step(id)?
        .ok_or_else(|| anyhow::anyhow!("unknown plan step {label}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use niucode_core::{now_ms, SessionLog};
    use niuse_core::Date;

    fn store() -> (tempfile::TempDir, Store) {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        (directory, store)
    }

    #[test]
    fn adding_and_listing_uses_stable_labels() {
        let (_directory, store) = store();
        let first = plan_add(&store, "write tests").unwrap();
        let second = plan_add(&store, "run clippy").unwrap();
        assert_eq!(first.label(), "p-1");
        assert_eq!(second.label(), "p-2");
        assert_eq!(store.plan().unwrap().len(), 2);
    }

    #[test]
    fn closing_requires_a_passing_verification() {
        let (directory, store) = store();
        let step = plan_add(&store, "write tests").unwrap();
        let thread = directory.path().join("thread.jsonl");
        let log = SessionLog::create(directory.path()).unwrap();
        std::fs::write(&thread, "").unwrap();
        let append = SessionLog::open(&thread).unwrap();
        append
            .append(&Event::Verification {
                command: "cargo test".into(),
                ok: false,
                output: "failed".into(),
                step: Some(step.label()),
                ts: now_ms(),
            })
            .unwrap();
        assert!(plan_done(&store, &thread, 1).is_err());

        log.append(&Event::Verification {
            command: "cargo test".into(),
            ok: true,
            output: "ok".into(),
            step: Some(step.label()),
            ts: now_ms(),
        })
        .unwrap();
        // The failing record lives in a different thread; this one passes.
        let passing_thread = log.path().to_path_buf();
        let closed = plan_done(&store, &passing_thread, 1).unwrap();
        assert!(closed.done);
        assert!(plan_done(&store, &passing_thread, 1).is_err());
        let _ = Date::today();
    }
}
