//! niuse's `TurnHooks` adapter for the niucode TUI.
//!
//! Prototype: not wired into a `niuse tui` command yet. The unit tests exercise
//! the full capture → parse → write path without a terminal.

use std::path::PathBuf;

use anyhow::Result;
use niucode::{AgentEvent, RunOutcome, TurnHooks, TurnPrompt};
use niucode_core::Event;
use niuse_core::candidate::{parse, Block, StreamFilter};
use niuse_core::date::Date;
use niuse_core::model::{LoopItem, MemoryEntry, MemoryKind};
use niuse_core::policy::secret_reason;
use niuse_core::prompt::compose;
use niuse_core::Store;

#[derive(Clone)]
enum Action {
    Accept,
    Replace(String),
    KeepBoth,
    Edit,
    Discard,
}

struct PendingMemory {
    kind: MemoryKind,
    tags: Vec<String>,
    text: String,
    actions: Vec<Action>,
}

pub struct CompanionHooks {
    root: PathBuf,
    prompt: String,
    filter: StreamFilter,
    raw: String,
    pending: Vec<Option<PendingMemory>>,
    turns: usize,
    input_tokens: u64,
    output_tokens: u64,
    cached_tokens: u64,
    memory_enabled: bool,
}

impl CompanionHooks {
    pub fn new(store: &Store) -> Result<Self> {
        let enabled = std::env::var("NIUSE_MEMORY")
            .map(|value| value.trim().to_lowercase() != "off")
            .unwrap_or(true);
        Self::with_memory(store, enabled)
    }

    fn with_memory(store: &Store, memory_enabled: bool) -> Result<Self> {
        let memories = if memory_enabled {
            store.memories()?
        } else {
            Vec::new()
        };
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
            turns: 0,
            input_tokens: 0,
            output_tokens: 0,
            cached_tokens: 0,
            memory_enabled,
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
        let memories = store.memories().unwrap_or_default();
        let mut prompts = Vec::new();
        self.pending.clear();

        for block in parsed.blocks {
            match block {
                Block::Journal { bullets } => {
                    if let Ok(true) = store.append_journal(Date::today(), &bullets) {
                        let _ = store.append_metric("journal", &bullets.len().to_string());
                    }
                }
                Block::Loop { due, text } => {
                    if let Ok(item) = store.add_loop(LoopItem::new(text, due)) {
                        let _ = store.append_metric("loop", &item.id);
                    }
                }
                Block::Memory { kind, tags, text } => {
                    if !self.memory_enabled {
                        continue;
                    }
                    if secret_reason(&text).is_some() {
                        let _ = store.append_metric("blocked", "secret");
                        continue;
                    }
                    let index = self.pending.len();
                    let tag_text = tags.join(",");
                    let conflict = memories.iter().find(|entry| {
                        niuse_core::policy::conflicts(&text, &tags, &entry.text, &entry.tags)
                    });
                    let (actions, options, prefix) = match conflict {
                        Some(entry) => (
                            vec![
                                Action::Replace(entry.id.clone()),
                                Action::KeepBoth,
                                Action::Edit,
                                Action::Discard,
                            ],
                            vec![
                                "replace".to_string(),
                                "keep both".to_string(),
                                "edit".to_string(),
                                "discard".to_string(),
                            ],
                            format!("conflicts with m-{} ({})", entry.id, entry.text),
                        ),
                        None => (
                            vec![Action::Accept, Action::Edit, Action::Discard],
                            vec![
                                "accept".to_string(),
                                "edit".to_string(),
                                "discard".to_string(),
                            ],
                            "new memory".to_string(),
                        ),
                    };
                    self.pending.push(Some(PendingMemory {
                        kind,
                        tags: tags.clone(),
                        text: text.clone(),
                        actions,
                    }));
                    let _ = store.append_metric("propose", &tag_text);
                    prompts.push(TurnPrompt {
                        id: format!("memory-{index}"),
                        question: format!(
                            "{} memory [{}] · {}: {}",
                            kind.as_str(),
                            if tag_text.is_empty() { "-" } else { &tag_text },
                            prefix,
                            text
                        ),
                        options,
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
        let Some(action) = memory.actions.get(choice).cloned() else {
            return;
        };
        let add = |text: String| {
            let entry = MemoryEntry::new(
                memory.kind,
                text,
                memory.tags.clone(),
                Date::today(),
                Some("tui-hook".into()),
            );
            let _ = store.add_memory(entry);
            let _ = store.append_metric("accept", "tui-hook");
        };
        match action {
            Action::Accept | Action::KeepBoth => add(memory.text),
            Action::Replace(id) => {
                let _ = store.update_memory(&id, |entry| {
                    entry.kind = memory.kind;
                    entry.text = memory.text.clone();
                    entry.tags = memory.tags.clone();
                    entry.confirmed = Date::today();
                });
                let _ = store.append_metric("accept", "tui-hook");
            }
            Action::Edit => {
                if let Some(edited) = crate::commands::edit_text(&memory.text) {
                    if secret_reason(&edited).is_none() {
                        add(edited);
                    } else {
                        let _ = store.append_metric("blocked", "secret");
                    }
                }
            }
            Action::Discard => {
                let _ = store.append_metric("discard", "tui-hook");
            }
        }
    }

    fn on_event(&mut self, _event: &AgentEvent) {}

    fn on_turn_end(&mut self, outcome: &RunOutcome) {
        self.turns += outcome.turns;
        self.input_tokens += outcome.usage.input_tokens;
        self.output_tokens += outcome.usage.output_tokens;
        self.cached_tokens += outcome.usage.cached_tokens;
    }

    fn commands(&self) -> Vec<(String, String)> {
        vec![
            ("/mem".into(), "list memory; /mem show <id>".into()),
            ("/loops".into(), "list open promises".into()),
            ("/journal".into(), "show the last three journal days".into()),
            ("/status".into(), "identity, memory, and loop counts".into()),
            ("/cost".into(), "session token usage".into()),
            ("/export".into(), "export the thread as Markdown".into()),
            ("/threads".into(), "list archived thread chapters".into()),
            ("/sessions".into(), "list one-shot session logs".into()),
        ]
    }

    fn command(&mut self, input: &str) -> Option<String> {
        let store = self.store();
        let mut parts = input.split_whitespace();
        let name = parts.next()?;
        match name {
            "/mem" => match parts.next() {
                None | Some("list") => {
                    let memories = store.memories().ok()?;
                    if memories.is_empty() {
                        return Some("(no memory)".into());
                    }
                    let mut out = String::new();
                    for entry in memories {
                        out.push_str(&format!(
                            "m-{} [{}] {}\n",
                            entry.id,
                            entry.kind.as_str(),
                            entry.text
                        ));
                    }
                    Some(out.trim_end().to_string())
                }
                Some("show") => {
                    let id = parts.next()?;
                    let id = id.strip_prefix("m-").unwrap_or(id);
                    let memories = store.memories().ok()?;
                    let entry = memories.iter().find(|entry| entry.id == id)?;
                    let tag_text = entry.tags.join(",");
                    Some(format!(
                        "m-{}\nkind: {}\ntags: {}\ncreated: {}\nconfirmed: {}\n{}",
                        entry.id,
                        entry.kind.as_str(),
                        if tag_text.is_empty() { "-" } else { &tag_text },
                        entry.created.format(),
                        entry.confirmed.format(),
                        entry.text
                    ))
                }
                _ => None,
            },
            "/loops" => {
                let loops = store.loops().ok()?;
                if loops.is_empty() {
                    return Some("(no open loops)".into());
                }
                let mut out = String::new();
                for item in loops.iter().filter(|item| !item.done) {
                    out.push_str(&format!("l-{} {}\n", item.id, item.text));
                }
                Some(out.trim_end().to_string())
            }
            "/journal" => {
                let days = store.recent_journal(3).ok()?;
                if days.is_empty() {
                    return Some("(journal is empty)".into());
                }
                let mut out = String::new();
                for day in days {
                    out.push_str(&format!("## {}\n", day.date.format()));
                    for bullet in day.bullets {
                        out.push_str(&format!("- {bullet}\n"));
                    }
                }
                Some(out.trim_end().to_string())
            }
            "/threads" => {
                let mut out = String::new();
                out.push_str(&format!("active: {}\n", store.session_path().display()));
                if let Ok(entries) = std::fs::read_dir(store.threads_dir()) {
                    let mut names: Vec<String> = entries
                        .flatten()
                        .filter_map(|entry| entry.file_name().into_string().ok())
                        .collect();
                    names.sort();
                    for name in names {
                        out.push_str(&format!("archived: {name}\n"));
                    }
                }
                Some(out.trim_end().to_string())
            }
            "/sessions" => {
                let mut names: Vec<String> = std::fs::read_dir(store.root().join("sessions"))
                    .ok()
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter_map(|entry| entry.file_name().into_string().ok())
                    .collect();
                names.sort();
                if names.is_empty() {
                    Some("(no sessions)".into())
                } else {
                    Some(names.join("\n"))
                }
            }
            "/export" => {
                let path = match parts.next() {
                    Some(path) => {
                        let path = PathBuf::from(path);
                        if path.is_absolute() {
                            path
                        } else {
                            std::env::current_dir().ok()?.join(path)
                        }
                    }
                    None => std::env::current_dir().ok()?.join("export-thread.md"),
                };
                let loaded = niucode_core::load(&self.root.join("thread.jsonl")).ok()?;
                let title = loaded
                    .events
                    .iter()
                    .find_map(|event| match event {
                        Event::UserMessage { text, .. } => {
                            Some(text.chars().take(60).collect::<String>().trim().to_string())
                        }
                        _ => None,
                    })
                    .filter(|text| !text.is_empty())
                    .unwrap_or_else(|| "niuse thread".to_string());
                let markdown = crate::export::render(&loaded.events, &title);
                crate::commands::write_export(&path, &markdown).ok()?;
                Some(format!("exported {}", path.display()))
            }
            "/cost" => Some(format!(
                "session: {} turns · {} in · {} out · {} cached",
                self.turns, self.input_tokens, self.output_tokens, self.cached_tokens
            )),
            "/status" => Some(format!(
                "identity: {}\nmemory: {}\nopen loops: {}",
                if store.identity().ok().flatten().is_some() {
                    "set"
                } else {
                    "missing"
                },
                store.memories().map(|entries| entries.len()).unwrap_or(0),
                store
                    .loops()
                    .map(|loops| loops.iter().filter(|item| !item.done).count())
                    .unwrap_or(0)
            )),
            _ => None,
        }
    }
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
            <<niuse:journal>>\n- learned something\n<</niuse:journal>>\n\
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
        let metrics = store.metrics().unwrap();
        assert!(metrics.contains("\tjournal\t"));
        assert!(metrics.contains("\tloop\t"));
    }

    fn seed(store: &Store) {
        store
            .add_memory(MemoryEntry::new(
                MemoryKind::Preference,
                "Rust projects use cargo test",
                vec!["rust".into()],
                Date::today(),
                None,
            ))
            .unwrap();
    }

    fn conflict_prompt(store: &Store) -> (CompanionHooks, TurnPrompt) {
        let mut hooks = CompanionHooks::new(store).unwrap();
        let mut text = "<<niuse:memory kind=preference tags=rust>>\nRust projects use cargo nextest\n<</niuse:memory>>".to_string();
        hooks.filter_text(&mut text);
        let prompts = hooks.after_turn(&outcome());
        assert_eq!(prompts.len(), 1);
        assert!(prompts[0].options.contains(&"replace".to_string()));
        (hooks, prompts[0].clone())
    }

    #[test]
    fn conflict_can_replace() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        seed(&store);
        let (mut hooks, prompt) = conflict_prompt(&store);
        hooks.on_answer(&prompt.id, 0);
        let memories = store.memories().unwrap();
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].text, "Rust projects use cargo nextest");
    }

    #[test]
    fn conflict_can_keep_both() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        seed(&store);
        let (mut hooks, prompt) = conflict_prompt(&store);
        hooks.on_answer(&prompt.id, 1);
        assert_eq!(store.memories().unwrap().len(), 2);
    }

    #[test]
    fn read_only_commands_read_the_store() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        seed(&store);
        let mut hooks = CompanionHooks::new(&store).unwrap();

        let listed = hooks.command("/mem").unwrap();
        assert!(listed.contains("Rust projects use cargo test"));
        let id = store.memories().unwrap()[0].id.clone();
        let shown = hooks.command(&format!("/mem show m-{id}")).unwrap();
        assert!(shown.contains("Rust projects use cargo test"));
        let status = hooks.command("/status").unwrap();
        assert!(status.contains("memory: 1"));
        assert!(hooks.command("/nope").is_none());
        let commands = hooks.commands();
        assert!(commands.iter().any(|(name, _)| name == "/mem"));

        let mut usage = outcome();
        usage.turns = 2;
        usage.usage.input_tokens = 123;
        usage.usage.output_tokens = 45;
        usage.usage.cached_tokens = 100;
        hooks.on_turn_end(&usage);
        let cost = hooks.command("/cost").unwrap();
        assert!(cost.contains("2 turns"));
        assert!(cost.contains("123 in"));
        assert!(cost.contains("100 cached"));
    }

    #[test]
    fn memory_off_suppresses_candidates_but_keeps_journal() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        let mut hooks = CompanionHooks::with_memory(&store, false).unwrap();
        let mut text =
            "<<niuse:memory kind=preference tags=rust>>\nno candidate\n<</niuse:memory>>\n\
            <<niuse:journal>>\n- still journaled\n<</niuse:journal>>"
                .to_string();
        hooks.filter_text(&mut text);
        let prompts = hooks.after_turn(&outcome());
        assert!(prompts.is_empty());
        assert!(store.memories().unwrap().is_empty());
        assert!(!store.recent_journal(1).unwrap().is_empty());
    }

    #[test]
    fn export_writes_the_thread() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        let log = niucode_core::SessionLog::create(store.root()).unwrap();
        log.append(&Event::UserMessage {
            text: "exported prompt".into(),
            ts: 1,
        })
        .unwrap();
        let source = log.path().to_path_buf();
        drop(log);
        std::fs::rename(source, store.session_path()).unwrap();

        let mut hooks = CompanionHooks::new(&store).unwrap();
        let target = directory.path().join("out.md");
        let message = hooks
            .command(&format!("/export {}", target.display()))
            .unwrap();
        assert!(message.contains("exported"));
        let markdown = std::fs::read_to_string(&target).unwrap();
        assert!(markdown.contains("exported prompt"));
    }

    #[test]
    fn lists_threads_and_sessions() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        std::fs::create_dir_all(store.threads_dir()).unwrap();
        std::fs::write(store.threads_dir().join("thread-1.jsonl"), "{}\n").unwrap();
        std::fs::create_dir_all(store.root().join("sessions")).unwrap();
        std::fs::write(store.root().join("sessions").join("s-1.jsonl"), "{}\n").unwrap();
        let mut hooks = CompanionHooks::new(&store).unwrap();
        assert!(hooks
            .command("/threads")
            .unwrap()
            .contains("thread-1.jsonl"));
        assert!(hooks.command("/sessions").unwrap().contains("s-1.jsonl"));
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
