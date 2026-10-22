//! `niuse evidence`: show what a thread actually ran and verified.

use std::path::Path;

use anyhow::Result;

use crate::evidence::from_thread;

pub fn evidence(thread: &Path, json: bool) -> Result<()> {
    let report = from_thread(thread)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print!("{}", report.render());
    }
    Ok(())
}
