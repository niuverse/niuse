//! Minimal terminal rendering for `niuse`. The niucode TUI lives behind a
//! binary-only crate, so niuse owns this small renderer and keeps the turn loop
//! as the single source of events.

use std::io::{self, Write};

use niucode_agent::{AgentEvent, ApprovalRequest};
use niuse_core::{parse, Parsed, StreamFilter};

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
                println!("→ {} {}", name, truncate(&input.to_string(), 180));
                self.at_line_start = true;
            }
            AgentEvent::ToolResult {
                name, ok, summary, ..
            } => {
                self.newline();
                let mark = if ok { "✓" } else { "✗" };
                println!("{mark} {name}: {}", truncate(&summary, 240));
                self.at_line_start = true;
            }
            AgentEvent::Warning(warning) => {
                self.newline();
                println!("! {warning}");
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
            AgentEvent::TurnEnd { .. } | AgentEvent::Phase(_) => {}
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
    println!("[approval] {}: {}", request.tool, request.summary);
    if let Some(preview) = &request.preview {
        if preview.added + preview.removed > 0 {
            println!("  +{} −{}", preview.added, preview.removed);
        }
        if let Some(warning) = &preview.warning {
            println!("  warning: {warning}");
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
