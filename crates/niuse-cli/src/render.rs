//! Minimal terminal rendering for `niuse`. The niucode TUI lives behind a
//! binary-only crate, so niuse owns this small renderer and keeps the turn loop
//! as the single source of events.

use std::env;
use std::io::{self, IsTerminal, Write};

use niucode_agent::{AgentEvent, ApprovalRequest};
use niucode_tools::{ChangePreview, LineKind, PreviewBody};
use niuse_core::{parse, Parsed, StreamFilter};

/// Dim, yellow, or red only on a real terminal that has not opted out.
fn styled(code: &str, text: &str) -> String {
    if io::stdout().is_terminal() && env::var_os("NO_COLOR").is_none() {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

/// Color code for one diff line: headers bold, additions green, removals
/// red, hunk headers cyan. Context lines are left alone.
fn diff_style(line: &str) -> Option<&'static str> {
    if line.starts_with("+++") || line.starts_with("---") {
        Some("1")
    } else if line.starts_with('+') {
        Some("32")
    } else if line.starts_with('-') {
        Some("31")
    } else if line.starts_with("@@") {
        Some("36")
    } else {
        None
    }
}

fn paint_diff(line: &str) -> String {
    match diff_style(line) {
        Some(code) => styled(code, line),
        None => line.to_string(),
    }
}

/// Short token counts: `999`, `1.5k`, `12k`.
pub(crate) fn compact_tokens(tokens: u64) -> String {
    if tokens < 1_000 {
        return tokens.to_string();
    }
    let value = tokens as f64 / 1_000.0;
    if value >= 10.0 {
        format!("{value:.0}k")
    } else {
        format!("{value:.1}k").replace(".0k", "k")
    }
}

pub struct Renderer {
    filter: StreamFilter,
    at_line_start: bool,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            filter: StreamFilter::new(),
            at_line_start: true,
        }
    }

    pub fn event(&mut self, event: AgentEvent) {
        match event {
            AgentEvent::TextDelta(delta) => {
                let visible = self.filter.push(&delta);
                if !visible.is_empty() {
                    print!("{visible}");
                    let _ = io::stdout().flush();
                    self.at_line_start = visible.ends_with('\n');
                }
            }
            AgentEvent::ReasoningDelta(_) => {}
            AgentEvent::ToolCall { name, input } => {
                self.newline();
                let line = format!("→ {} {}", name, truncate(&input.to_string(), 180));
                println!("{}", styled("2", &line));
                self.at_line_start = true;
            }
            AgentEvent::ToolResult {
                name, ok, summary, ..
            } => {
                self.newline();
                let mark = if ok { "✓" } else { "✗" };
                let line = format!("{mark} {name}: {}", truncate(&summary, 240));
                let line = if ok {
                    styled("2", &line)
                } else {
                    styled("31", &line)
                };
                println!("{line}");
                self.at_line_start = true;
            }
            AgentEvent::Warning(warning) => {
                self.newline();
                println!("{}", styled("33", &format!("! {warning}")));
                self.at_line_start = true;
            }
            AgentEvent::Compaction {
                before_tokens,
                after_tokens,
                ..
            } => {
                self.newline();
                println!("[compacted {before_tokens} → {after_tokens} tokens]");
                self.at_line_start = true;
            }
            AgentEvent::TurnEnd { usage } => {
                self.newline();
                let mut line = format!(
                    "  {} in · {} out",
                    compact_tokens(usage.input_tokens),
                    compact_tokens(usage.output_tokens)
                );
                if usage.cached_tokens > 0 {
                    line.push_str(&format!(
                        " · {} cached",
                        compact_tokens(usage.cached_tokens)
                    ));
                }
                println!("{}", styled("2", &line));
                self.at_line_start = true;
            }
            AgentEvent::Phase(_) => {}
        }
    }

    /// Flush held text, print a final newline, and return the parsed answer.
    pub fn finish(&mut self) -> Parsed {
        let (tail, raw) = self.filter.finish();
        if !tail.is_empty() {
            print!("{tail}");
            let _ = io::stdout().flush();
            self.at_line_start = tail.ends_with('\n');
        }
        self.newline();
        parse(&raw)
    }

    fn newline(&mut self) {
        if !self.at_line_start {
            println!();
            let _ = io::stdout().flush();
            self.at_line_start = true;
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-action approval for mutating tools. Reads one line from the terminal.
pub fn approve(request: &ApprovalRequest) -> bool {
    println!();
    println!(
        "{}",
        styled(
            "1",
            &format!("[approval] {}: {}", request.tool, request.summary)
        )
    );
    if let Some(preview) = &request.preview {
        for line in preview_lines(preview, 80) {
            println!("{}", paint_diff(&line));
        }
        if let Some(warning) = &preview.warning {
            println!("{}", styled("33", &format!("  warning: {warning}")));
        }
    }
    print!("allow? [y/N] ");
    let _ = io::stdout().flush();

    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        return false;
    }
    matches!(line.trim().to_lowercase().as_str(), "y" | "yes")
}

/// Read one line from stdin; `None` on EOF.
pub fn read_line() -> Option<String> {
    let mut line = String::new();
    match io::stdin().read_line(&mut line) {
        Ok(0) => None,
        Ok(_) => Some(line.trim_end().to_string()),
        Err(_) => None,
    }
}

/// Render the same structured preview niucode's TUI shows, capped so a large
/// edit cannot swallow the terminal. The prompt stays after the diff.
fn preview_lines(preview: &ChangePreview, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    match &preview.body {
        PreviewBody::Diff { files } => {
            for (index, file) in files.iter().enumerate() {
                if index > 0 {
                    lines.push(String::new());
                }
                let marker = if file.is_new { " (new)" } else { "" };
                lines.push(format!("--- {} {}", file.path, marker));
                for hunk in &file.hunks {
                    lines.push(format!(
                        "@@ -{},{} +{},{} @@",
                        hunk.old_start, hunk.old_len, hunk.new_start, hunk.new_len
                    ));
                    for line in &hunk.lines {
                        let prefix = match line.kind {
                            LineKind::Context => ' ',
                            LineKind::Add => '+',
                            LineKind::Remove => '-',
                        };
                        lines.push(format!("{prefix}{}", line.text.trim_end_matches('\n')));
                    }
                }
            }
        }
        PreviewBody::Command { command } => lines.push(format!("$ {command}")),
        PreviewBody::Note { message } => lines.push(message.clone()),
    }

    if lines.len() > max_lines {
        let omitted = lines.len() - max_lines;
        lines.truncate(max_lines);
        lines.push(format!("… {omitted} more line(s)"));
    }
    lines
}

fn truncate(text: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (count, character) in text.chars().enumerate() {
        if count >= max_chars {
            out.push('…');
            break;
        }
        if character == '\n' {
            out.push(' ');
        } else {
            out.push(character);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use niucode_tools::{ChangePreview, FilePreview, PreviewBody};

    #[test]
    fn diff_preview_shows_hunks_and_marks_additions() {
        let file = FilePreview::from_text("a.txt", "alpha\nbeta\n", "alpha\ngamma\n", false);
        let preview = ChangePreview {
            added: file.added,
            removed: file.removed,
            warning: None,
            body: PreviewBody::Diff { files: vec![file] },
        };
        let lines = preview_lines(&preview, 80);
        assert!(lines.iter().any(|line| line == "-beta"));
        assert!(lines.iter().any(|line| line == "+gamma"));
        assert!(lines.iter().any(|line| line.starts_with("@@ ")));
    }

    #[test]
    fn long_previews_are_capped() {
        let old = (0..200)
            .map(|i| format!("old {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let new = (0..200)
            .map(|i| format!("new {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let file = FilePreview::from_text("big.txt", &old, &new, false);
        let preview = ChangePreview {
            added: file.added,
            removed: file.removed,
            warning: None,
            body: PreviewBody::Diff { files: vec![file] },
        };
        let lines = preview_lines(&preview, 10);
        assert_eq!(lines.len(), 11);
        assert!(lines.last().unwrap().contains("more line"));
    }
}
