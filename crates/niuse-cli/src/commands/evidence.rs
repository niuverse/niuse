//! `niuse evidence`: show what a thread actually ran and verified.

use std::path::Path;

use anyhow::Result;

use niuse_core::Store;

use crate::evidence::from_thread;

pub fn evidence(thread: &Path, store: &Store, json: bool) -> Result<()> {
    let mut report = from_thread(thread)?;
    report.plan = store.plan()?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print!("{}", report.render());
    }
    Ok(())
}
