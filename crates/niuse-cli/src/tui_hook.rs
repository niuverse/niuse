//! niuse's `TurnHooks` adapter for the niucode TUI.
//!
//! Prototype: not wired into a `niuse tui` command yet. The unit tests exercise
//! the full capture → parse → write path without a terminal.

use std::path::PathBuf;

use anyhow::Result;
use niucode::{AgentEvent, RunOutcome, TurnHooks, TurnPrompt};
use niuse_core::candidate::{parse, Block, StreamFilter};
use niuse_core::date::Date;
use niuse_core::model::{LoopItem, MemoryEntry, MemoryKind};
use niuse_core::policy::secret_reason;
use niuse_core::prompt::compose;
use niuse_core::Store;

struct PendingMemory {
    kind: MemoryKind,
    tags: Vec<String>,
    text: String,
}

pub struct CompanionHooks {
    root: PathBuf,
    prompt: String,
    filter: StreamFilter,
    raw: String,
    pending: Vec<Option<PendingMemory>>,
}

impl CompanionHooks {
    pub fn new(store: &Store) -> Result<Self> {
        let memories = store.memories()?;
        let loops = store.loops()?;
        let journal = store.recent_journal(3)?;
        let identity = store.identity()?;
        let user = store.user()?;
        let composition = compose(
            identity.as_deref(),
            user.as_deref(),
            &memories,
            &loops,
            &journal,
            Date::today(),
        );
        Ok(Self {
            root: store.root().to_path_buf(),
            prompt: composition.system,
            filter: StreamFilter::new(),
            raw: String::new(),
            pending: Vec::new(),
        })
    }

    fn store(&self) -> Store {
        Store::at(self.root.clone())
    }
}

impl TurnHooks for CompanionHooks {
    fn augment_system_prompt(&mut self, prompt: &mut String) {
        prompt.push_str("\n\n");
        prompt.push_str(&self.prompt);
    }

    fn filter_text(&mut self, text: &mut String) {
        self.raw.push_str(text);
        *text = self.filter.push(text);
    }

    fn after_turn(&mut self, _outcome: &RunOutcome) -> Vec<TurnPrompt> {
        let raw = std::mem::take(&mut self.raw);
        self.filter = StreamFilter::new();
        let parsed = parse(&raw);
        let store = self.store();
        let mut prompts = Vec::new();
        self.pending.clear();

        for block in parsed.blocks {
            match block {
                Block::Journal { bullets } => {
                    let _ = store.append_journal(Date::today(), &bullets);
                }
                Block::Loop { due, text } => {
                    let _ = store.add_loop(LoopItem::new(text, due));
                }
                Block::Memory { kind, tags, text } => {
                    if secret_reason(&text).is_some() {
                        let _ = store.append_metric("blocked", "secret");
                        continue;
                    }
                    let index = self.pending.len();
                    self.pending.push(Some(PendingMemory {
                        kind,
                        tags: tags.clone(),
                        text: text.clone(),
                    }));
                    let tag_text = tags.join(",");
                    let _ = store.append_metric("propose", &tag_text);
                    prompts.push(TurnPrompt {
                        id: format!("memory-{index}"),
                        question: format!(
                            "{} memory [{}]: {}",
                            kind.as_str(),
                            if tag_text.is_empty() { "-" } else { &tag_text },
                            text
                        ),
                        options: vec!["accept".into(), "discard".into()],
                    });
                }
            }
        }
        prompts
    }

    fn on_answer(&mut self, id: &str, choice: usize) {
        let Some(index) = id
            .strip_prefix("memory-")
            .and_then(|value| value.parse::<usize>().ok())
        else {
            return;
        };
        let Some(slot) = self.pending.get_mut(index) else {
            return;
        };
        let Some(memory) = slot.take() else {
            return;
        };
        let store = self.store();
        match choice {
            0 => {
                let entry = MemoryEntry::new(
                    memory.kind,
                    memory.text,
                    memory.tags,
                    Date::today(),
                    Some("tui-hook".into()),
                );
                let _ = store.add_memory(entry);
                let _ = store.append_metric("accept", "tui-hook");
            }
            _ => {
                let _ = store.append_metric("discard", "tui-hook");
            }
        }
    }

    fn on_event(&mut self, _event: &AgentEvent) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use niucode_core::Usage;

    fn outcome() -> RunOutcome {
        RunOutcome {
            turns: 1,
            usage: Usage {
                input_tokens: 1,
                output_tokens: 1,
                cached_tokens: 0,
                reasoning_tokens: 0,
            },
            final_text: String::new(),
        }
    }

    #[test]
    fn strips_protocol_and_commits_on_accept() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        let mut hooks = CompanionHooks::new(&store).unwrap();
        let mut text = "hello\n\
            <<niuse:memory kind=preference tags=rust>>\nuse nextest\n<</niuse:memory>>\n\
            <<niuse:loop>>\ncheck the docs\n<</niuse:loop>>"
            .to_string();
        hooks.filter_text(&mut text);
        assert!(!text.contains("<<niuse:"));
        assert!(text.contains("hello"));

        let prompts = hooks.after_turn(&outcome());
        assert_eq!(prompts.len(), 1);
        assert_eq!(store.loops().unwrap().len(), 1);
        hooks.on_answer(&prompts[0].id, 0);

        let memories = store.memories().unwrap();
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].text, "use nextest");
    }

    #[test]
    fn discard_writes_no_memory() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        let mut hooks = CompanionHooks::new(&store).unwrap();
        let mut text =
            "<<niuse:memory kind=lesson tags=rust>>\nnever panic\n<</niuse:memory>>".to_string();
        hooks.filter_text(&mut text);
        let prompts = hooks.after_turn(&outcome());
        hooks.on_answer(&prompts[0].id, 1);
        assert!(store.memories().unwrap().is_empty());
    }
}
