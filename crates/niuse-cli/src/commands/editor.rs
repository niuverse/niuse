//! Shared `$EDITOR` helpers.
//!
//! Memory text is normalised to one line; identity, user, and other Markdown
//! bodies keep their shape.

use std::fs;

use niuse_core::model::one_line;

pub(crate) fn edit_markdown(initial: &str) -> Option<String> {
    let path = std::env::temp_dir().join(format!("niuse-edit-{}.md", std::process::id()));
    fs::write(&path, initial).ok()?;
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = std::process::Command::new(&editor).arg(&path).status();
    let text = fs::read_to_string(&path).ok();
    let _ = fs::remove_file(&path);
    status.ok()?.success().then_some(())?;
    text.map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

/// One-line variant used by candidate and memory editing.
pub(crate) fn edit_text(initial: &str) -> Option<String> {
    edit_markdown(initial)
        .map(one_line)
        .filter(|text| !text.is_empty())
}
