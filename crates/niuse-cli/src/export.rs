//! Markdown export of a session log.
//!
//! The event log is the source of truth; this is a lossy, human-readable view
//! for backup, review, or sharing. Tool output is clipped so one large command
//! cannot dominate the file.

use std::fmt::Write as _;

use niucode_core::Event;
use niuse_core::parse;

const MAX_TOOL_OUTPUT_CHARS: usize = 4_000;

pub fn render(events: &[Event], title: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# {title}\n");

    let mut wrote_meta = false;
    for event in events {
        match event {
            Event::SessionStart {
                cwd,
                model,
                provider,
                ..
            } => {
                let provider = if provider.is_empty() {
                    "unknown"
                } else {
                    provider.as_str()
                };
                let _ = writeln!(
                    out,
                    "> cwd `{cwd}` · model `{model}` · provider `{provider}`\n"
                );
                wrote_meta = true;
            }
            Event::UserMessage { text, .. } => {
                let _ = writeln!(out, "## user\n\n{}\n", text.trim());
            }
            Event::AssistantMessage {
                text, tool_calls, ..
            } => {
                // The log keeps protocol blocks in the raw answer; the export is
                // for humans, so strip them the same way the REPL does.
                let visible = parse(text).visible;
                if !visible.trim().is_empty() {
                    let _ = writeln!(out, "## assistant\n\n{}\n", visible.trim());
                }
                for call in tool_calls {
                    let input = serde_json::to_string_pretty(&call.input).unwrap_or_default();
                    let _ = writeln!(
                        out,
                        "### tool call · {}\n\n```json\n{input}\n```\n",
                        call.name
                    );
                }
            }
            Event::ToolResult { ok, output, .. } => {
                let status = if *ok { "ok" } else { "error" };
                let _ = writeln!(
                    out,
                    "### tool result · {status}\n\n```\n{}\n```\n",
                    clip(output, MAX_TOOL_OUTPUT_CHARS)
                );
            }
            Event::Compaction {
                upto,
                before_tokens,
                after_tokens,
                ..
            } => {
                let _ = writeln!(
                    out,
                    "> compaction: events[..{upto}] replaced, {before_tokens} → {after_tokens} tokens\n"
                );
            }
            Event::SessionEnd { .. } => {
                let _ = writeln!(out, "---\n\n_end of session_\n");
            }
            _ => {}
        }
    }

    if !wrote_meta {
        out.push_str("> no session metadata found\n\n");
    }
    out
}

fn clip(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut clipped: String = text.chars().take(max_chars).collect();
    clipped.push_str("\n…（已截断）");
    clipped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(text: &str) -> Event {
        Event::UserMessage {
            text: text.into(),
            ts: 0,
        }
    }

    fn assistant(text: &str) -> Event {
        Event::AssistantMessage {
            text: text.into(),
            reasoning: None,
            tool_calls: Vec::new(),
            ts: 0,
        }
    }

    #[test]
    fn renders_turns_in_order() {
        let events = vec![user("hello"), assistant("hi there")];
        let text = render(&events, "thread");
        let user_at = text.find("## user").unwrap();
        let assistant_at = text.find("## assistant").unwrap();
        assert!(user_at < assistant_at);
        assert!(text.contains("hello"));
        assert!(text.contains("hi there"));
    }

    #[test]
    fn protocol_blocks_do_not_leak_into_the_export() {
        let events = vec![assistant(
            "answer\n\n<<niuse:memory tags=rust>>\nremember this\n<</niuse:memory>>",
        )];
        let text = render(&events, "thread");
        assert!(text.contains("answer"));
        assert!(!text.contains("<<niuse:memory"));
        assert!(!text.contains("remember this"));
    }

    #[test]
    fn clips_long_tool_output() {
        let events = vec![Event::ToolResult {
            call_id: "c1".into(),
            ok: true,
            output: "x".repeat(MAX_TOOL_OUTPUT_CHARS + 10),
            ts: 0,
        }];
        let text = render(&events, "thread");
        assert!(text.contains("已截断"));
        assert!(text.len() < MAX_TOOL_OUTPUT_CHARS + 200);
    }

    #[test]
    fn empty_log_still_has_a_header() {
        let text = render(&[], "thread");
        assert!(text.starts_with("# thread"));
    }
}
