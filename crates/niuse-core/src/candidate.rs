//! In-band protocol blocks.
//!
//! The model may end an answer with `<<niuse:memory>>`, `<<niuse:loop>>`, or
//! `<<niuse:journal>>` blocks. They are stripped before rendering and turned
//! into durable records. No second model call is involved.

use crate::date::Date;
use crate::model::{one_line, MemoryKind};

pub const MARKER: &str = "<<niuse:";
const CLOSE_PREFIX: &str = "<</niuse:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Memory {
        kind: MemoryKind,
        tags: Vec<String>,
        text: String,
    },
    Loop {
        due: Option<Date>,
        text: String,
    },
    Journal {
        bullets: Vec<String>,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Parsed {
    pub visible: String,
    pub blocks: Vec<Block>,
}

/// Parse every complete block, returning the text the user should see.
pub fn parse(text: &str) -> Parsed {
    let mut visible = String::new();
    let mut blocks = Vec::new();
    let mut rest = text;

    loop {
        let Some(start) = rest.find(MARKER) else {
            visible.push_str(rest);
            break;
        };
        visible.push_str(&rest[..start]);
        let candidate = &rest[start..];
        let complete = candidate.find(">>").and_then(|header_end| {
            let body_start = header_end + 2;
            let close_rel = candidate[body_start..].find(CLOSE_PREFIX)?;
            let close_start = body_start + close_rel;
            let close_end = candidate[close_start..].find(">>")? + close_start + 2;
            Some((header_end, body_start, close_start, close_end))
        });

        let Some((header_end, body_start, close_start, close_end)) = complete else {
            // An unterminated block is model output, not a protocol message:
            // show it rather than silently eating the answer.
            visible.push_str(candidate);
            break;
        };

        if let Some(block) = parse_block(
            &candidate[..header_end + 2],
            &candidate[body_start..close_start],
        ) {
            blocks.push(block);
        }
        rest = &candidate[close_end..];
    }

    Parsed {
        visible: visible.trim_end().to_string(),
        blocks,
    }
}

fn parse_block(header: &str, body: &str) -> Option<Block> {
    let inner = header.strip_prefix(MARKER)?.strip_suffix(">>")?;
    let mut parts = inner.split_whitespace();
    let name = parts.next()?;
    let mut attributes = std::collections::HashMap::new();
    for part in parts {
        if let Some((key, value)) = part.split_once('=') {
            attributes.insert(key, value);
        }
    }

    match name {
        "memory" => {
            let kind = attributes
                .get("kind")
                .and_then(|value| MemoryKind::parse(value))
                .unwrap_or(MemoryKind::Preference);
            let tags = attributes
                .get("tags")
                .map(|value| {
                    value
                        .split(',')
                        .map(str::trim)
                        .filter(|tag| !tag.is_empty())
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();
            let text = one_line(body.to_string());
            (!text.is_empty()).then_some(Block::Memory { kind, tags, text })
        }
        "loop" => {
            let text = one_line(body.to_string());
            let due = attributes.get("due").and_then(|value| Date::parse(value));
            (!text.is_empty()).then_some(Block::Loop { due, text })
        }
        "journal" => {
            let bullets: Vec<String> = body
                .lines()
                .map(|line| one_line(line.trim_start_matches(['-', '*']).to_string()))
                .filter(|line| !line.is_empty())
                .take(5)
                .collect();
            (!bullets.is_empty()).then_some(Block::Journal { bullets })
        }
        _ => None,
    }
}

/// Streaming view of the model output.
///
/// Text is released as soon as it cannot be the prefix of a protocol block.
/// Complete blocks are swallowed; `finish` returns the full raw text so the
/// caller can parse the blocks without having kept a second buffer.
#[derive(Debug, Default)]
pub struct StreamFilter {
    buffer: String,
    raw: String,
}

impl StreamFilter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed one delta, returning visible text that is safe to print now.
    pub fn push(&mut self, delta: &str) -> String {
        self.buffer.push_str(delta);
        self.raw.push_str(delta);

        let mut visible = String::new();
        loop {
            if let Some(start) = self.buffer.find(MARKER) {
                if start > 0 {
                    visible.push_str(&self.buffer[..start]);
                    self.buffer.drain(..start);
                }
                let Some(header_end) = self.buffer.find(">>") else {
                    break;
                };
                let body_start = header_end + 2;
                let Some(close_rel) = self.buffer[body_start..].find(CLOSE_PREFIX) else {
                    break;
                };
                let close_start = body_start + close_rel;
                let Some(close_tail) = self.buffer[close_start..].find(">>") else {
                    break;
                };
                let close_end = close_start + close_tail + 2;
                self.buffer.drain(..close_end);
                continue;
            }

            let hold = held_suffix_len(&self.buffer);
            let flush = self.buffer.len() - hold;
            visible.push_str(&self.buffer[..flush]);
            self.buffer.drain(..flush);
            break;
        }
        visible
    }

    /// Flush the held tail and return `(tail, raw_text)`.
    pub fn finish(&mut self) -> (String, String) {
        let tail = std::mem::take(&mut self.buffer);
        let raw = std::mem::take(&mut self.raw);
        (tail, raw)
    }
}

/// How many trailing bytes could still grow into `<<niuse:`.
fn held_suffix_len(buffer: &str) -> usize {
    let max = MARKER.len().min(buffer.len());
    for length in (1..=max).rev() {
        let start = buffer.len() - length;
        if buffer.is_char_boundary(start) && MARKER.starts_with(&buffer[start..]) {
            return length;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_passes_through() {
        assert_eq!(parse("hello").visible, "hello");
        assert!(parse("hello").blocks.is_empty());
    }

    #[test]
    fn memory_block_is_stripped_and_parsed() {
        let text = "先看这里。\n\n<<niuse:memory kind=lesson tags=rust,testing>>\n用 nextest\n<</niuse:memory>>\n";
        let parsed = parse(text);
        assert_eq!(parsed.visible, "先看这里。");
        assert_eq!(
            parsed.blocks,
            vec![Block::Memory {
                kind: MemoryKind::Lesson,
                tags: vec!["rust".into(), "testing".into()],
                text: "用 nextest".into(),
            }]
        );
    }

    #[test]
    fn loop_and_journal_parse() {
        let text = concat!(
            "好的\n",
            "<<niuse:loop due=2026-10-22>>\n确认 cwd\n<</niuse:loop>>\n",
            "<<niuse:journal>>\n- 决定 A\n- 未解 B\n<</niuse:journal>>\n",
        );
        let parsed = parse(text);
        assert_eq!(parsed.visible, "好的");
        assert_eq!(
            parsed.blocks,
            vec![
                Block::Loop {
                    due: Date::parse("2026-10-22"),
                    text: "确认 cwd".into(),
                },
                Block::Journal {
                    bullets: vec!["决定 A".into(), "未解 B".into()],
                },
            ]
        );
    }

    #[test]
    fn malformed_block_is_kept_as_text() {
        let text = "开头\n<<niuse:memory>>没有结束";
        let parsed = parse(text);
        assert!(parsed.blocks.is_empty());
        assert_eq!(parsed.visible, text);
    }

    #[test]
    fn stream_holds_partial_marker() {
        let mut filter = StreamFilter::new();
        assert_eq!(filter.push("你好 <"), "你好 ");
        assert_eq!(filter.push("<niu"), "");
        assert_eq!(filter.push("se:memory>>x"), "");
        assert_eq!(filter.push("<</niuse:memory>> 完"), " 完");
        let (tail, raw) = filter.finish();
        assert_eq!(tail, "");
        assert_eq!(raw, "你好 <<niuse:memory>>x<</niuse:memory>> 完");
    }

    #[test]
    fn stream_releases_text_that_only_looks_like_a_marker() {
        let mut filter = StreamFilter::new();
        assert_eq!(filter.push("<"), "");
        assert_eq!(filter.push("b"), "<b");
    }

    #[test]
    fn stream_handles_block_split_across_deltas() {
        let mut filter = StreamFilter::new();
        let mut visible = String::new();
        for delta in [
            "结论",
            "\n<<niuse:me",
            "mory>>\n用 nextest\n<<",
            "/niuse:memory>>",
            "\n完",
        ] {
            visible.push_str(&filter.push(delta));
        }
        let (tail, raw) = filter.finish();
        visible.push_str(&tail);
        assert_eq!(visible, "结论\n\n完");
        assert_eq!(parse(&raw).blocks.len(), 1);
    }
}
