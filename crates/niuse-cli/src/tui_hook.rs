//! niuse's `TurnHooks` adapter for the niucode TUI.
//!
//! Prototype: not wired into a `niuse tui` command yet. The unit tests exercise
//! the full capture → parse → write path without a terminal.

use std::path::{Path, PathBuf};

use anyhow::Result;
use niucode::{AgentEvent, RunOutcome, TurnHooks, TurnPrompt};
use niucode_core::{load, Event};
use niuse_core::candidate::{parse, Block, StreamFilter};
use niuse_core::date::Date;
use niuse_core::model::{LoopItem, MemoryEntry, MemoryKind};
use niuse_core::policy::secret_reason;
use niuse_core::prompt::compose_with_plan;
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
    thread: PathBuf,
    cwd: PathBuf,
    prompt: String,
    filter: StreamFilter,
    raw: String,
    pending: Vec<Option<PendingMemory>>,
    memory_enabled: bool,
}

impl CompanionHooks {
    #[cfg(test)]
    pub fn new(store: &Store) -> Result<Self> {
        Self::new_with_thread(store, store.session_path())
    }

    pub fn new_with_thread(store: &Store, thread: PathBuf) -> Result<Self> {
        let enabled = std::env::var("NIUSE_MEMORY")
            .map(|value| value.trim().to_lowercase() != "off")
            .unwrap_or(true);
        Self::with_memory(store, enabled, thread)
    }

    fn with_memory(store: &Store, memory_enabled: bool, thread: PathBuf) -> Result<Self> {
        let memories = if memory_enabled {
            store.memories()?
        } else {
            Vec::new()
        };
        let loops = store.loops()?;
        let goals = store.goals()?;
        let plan = store.plan()?;
        let journal = store.recent_journal(3)?;
        let identity = store.identity()?;
        let user = store.user()?;
        let composition = compose_with_plan(
            identity.as_deref(),
            user.as_deref(),
            &goals,
            &plan,
            &memories,
            &loops,
            &journal,
            Date::today(),
        );
        Ok(Self {
            root: store.root().to_path_buf(),
            thread,
            cwd: std::env::current_dir().unwrap_or_default(),
            prompt: composition.system,
            filter: StreamFilter::new(),
            raw: String::new(),
            pending: Vec::new(),
            memory_enabled,
        })
    }

    fn store(&self) -> Store {
        Store::at(self.root.clone())
    }

    pub fn with_cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = cwd;
        self
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

    fn on_turn_end(&mut self, outcome: &RunOutcome) {
        let _ = std::fs::write(
            self.store().root().join("on-turn-debug"),
            format!("{:?}\n", outcome.usage),
        );
        let store = self.store();
        let usage = &outcome.usage;
        if usage.input_tokens + usage.output_tokens + usage.cached_tokens > 0 {
            let _ = store.append_metric(
                "tokens",
                &format!(
                    "{},{},{}",
                    usage.input_tokens, usage.output_tokens, usage.cached_tokens
                ),
            );
        }
        let _ = store.append_metric("turn", "ok");
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

    fn new_session_path(&mut self) -> Result<Option<PathBuf>> {
        let threads = self.root.join("threads");
        crate::driver::archive_thread(&self.thread, &threads)?;
        Ok(Some(self.root.join("thread.jsonl")))
    }

    fn on_session_start(&mut self, path: &Path) {
        self.thread = path.to_path_buf();
    }

    fn commands(&self) -> Vec<(String, String)> {
        vec![
            ("/brief".into(), "what needs attention today".into()),
            (
                "/evidence".into(),
                "tools, files, checks, and failures".into(),
            ),
            (
                "/report".into(),
                "weekly review: turns, cost, stores, dogfood".into(),
            ),
            (
                "/goals".into(),
                "list goals; /goals add <text>; /goals done <id>".into(),
            ),
            (
                "/steps".into(),
                "list steps; /steps add <text>; /steps done <p-N>".into(),
            ),
            (
                "/verify".into(),
                "run a check; /verify p-N <command> tags a plan step".into(),
            ),
            ("/recall".into(), "search memory, journal, and loops".into()),
            ("/mem".into(), "list memory; /mem show <id>".into()),
            ("/loops".into(), "list open promises".into()),
            ("/journal".into(), "show the last three journal days".into()),
            ("/status".into(), "identity, memory, and loop counts".into()),
            ("/cost".into(), "session token usage".into()),
            ("/export".into(), "export the thread as Markdown".into()),
            (
                "/threads".into(),
                "list or filter archived thread chapters".into(),
            ),
            (
                "/sessions".into(),
                "list or filter one-shot session logs".into(),
            ),
            ("/prompt".into(), "print the composed system prompt".into()),
            ("/identity".into(), "print identity.md".into()),
            ("/user".into(), "print user.md".into()),
        ]
    }

    fn command(&mut self, input: &str) -> Option<String> {
        let store = self.store();
        let mut parts = input.split_whitespace();
        let name = parts.next()?;
        match name {
            "/brief" => crate::commands::brief_text(&store, &self.cwd).ok(),
            "/evidence" => {
                let mut report = crate::evidence::from_thread(&self.thread).ok()?;
                report.plan = store.plan().ok()?;
                Some(report.render())
            }
            "/report" => {
                let dogfood = self.cwd.join("docs/dogfood.md");
                let dogfood = dogfood.is_file().then_some(dogfood);
                match crate::commands::report_text(&store, 7, dogfood) {
                    Ok(text) => Some(text),
                    Err(error) => Some(format!("error: {error:#}")),
                }
            }
            "/steps" => match parts.next() {
                None | Some("list") => {
                    let report = crate::evidence::from_thread(&self.thread).unwrap_or_default();
                    let steps = store.plan().ok()?;
                    let open: Vec<String> = steps
                        .iter()
                        .filter(|step| !step.done)
                        .map(|step| {
                            format!(
                                "[ ] {} {} — {}",
                                step.label(),
                                step.text,
                                report.step_state(step)
                            )
                        })
                        .collect();
                    if open.is_empty() {
                        Some("no open steps".to_string())
                    } else {
                        Some(open.join("\n"))
                    }
                }
                Some("add") => {
                    let text = input
                        .split_once("/steps add")
                        .map(|(_, rest)| rest.trim())
                        .unwrap_or("");
                    match crate::commands::plan_add(&store, text) {
                        Ok(step) => Some(format!("added {}", step.label())),
                        Err(error) => Some(format!("error: {error:#}")),
                    }
                }
                Some("done") => {
                    let Some(id) = parts
                        .next()
                        .and_then(|id| id.trim_start_matches("p-").parse::<u32>().ok())
                    else {
                        return Some("usage: /steps done <N>".to_string());
                    };
                    match crate::commands::plan_done(&store, &self.thread, id) {
                        Ok(step) => Some(format!("done {}", step.label())),
                        Err(error) => Some(format!("error: {error:#}")),
                    }
                }
                _ => Some("usage: /steps [add <text>|done <N>]".to_string()),
            },
            "/goals" => match parts.next() {
                None | Some("list") => Some(
                    store
                        .goals()
                        .ok()?
                        .iter()
                        .filter(|goal| !goal.done)
                        .map(|goal| goal.render())
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
                .filter(|text| !text.is_empty())
                .or_else(|| Some("(no open goals)".into())),
                Some("add") => {
                    let text = parts.collect::<Vec<_>>().join(" ");
                    crate::commands::goals_add_one(&store, &text)
                        .ok()
                        .map(|goal| format!("goal g-{}: {}", goal.id, goal.text))
                }
                Some("done") => {
                    let id = parts.next()?;
                    crate::commands::goals_close(&store, id)
                        .ok()
                        .map(|id| format!("goal g-{id} done"))
                }
                _ => None,
            },
            "/verify" => {
                let rest = input.strip_prefix("/verify").unwrap_or("").trim();
                if rest.is_empty() {
                    return Some("usage: /verify [p-N] <command>".to_string());
                }
                let (step, command) = match rest.split_once(' ') {
                    Some((first, command)) if first.starts_with("p-") && !command.is_empty() => {
                        (Some(first), command.trim())
                    }
                    _ => (None, rest),
                };
                match crate::commands::verify_shell_with_step(&self.thread, command, step, false) {
                    Ok(outcome) => Some(format!(
                        "verify · {} · {command}",
                        if outcome.ok { "passed" } else { "FAILED" }
                    )),
                    Err(error) => Some(format!("error: {error:#}")),
                }
            }
            "/recall" => {
                let query = parts.collect::<Vec<_>>().join(" ");
                if query.is_empty() {
                    Some("usage: /recall <query>".into())
                } else {
                    crate::commands::recall_text(&store, &query, 20).ok()
                }
            }
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
            "/prompt" => Some(self.prompt.clone()),
            "/identity" => Some(
                store
                    .identity()
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| "(missing; run `niuse identity init`)".into()),
            ),
            "/user" => Some(
                store
                    .user()
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| "(missing; run `niuse user edit`)".into()),
            ),
            "/threads" => {
                let query = parts.collect::<Vec<_>>().join(" ").to_lowercase();
                let mut out = String::new();
                if query.is_empty()
                    || self
                        .thread
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&query)
                {
                    out.push_str(&format!("active: {}\n", self.thread.display()));
                }
                if let Ok(entries) = std::fs::read_dir(store.threads_dir()) {
                    let mut paths: Vec<PathBuf> =
                        entries.flatten().map(|entry| entry.path()).collect();
                    paths.sort();
                    for path in paths {
                        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                            continue;
                        };
                        let title = first_user_title(&path);
                        let hay = format!(
                            "{} {}",
                            name.to_lowercase(),
                            title.as_deref().unwrap_or("").to_lowercase()
                        );
                        if query.split_whitespace().all(|needle| hay.contains(needle)) {
                            match title {
                                Some(title) => {
                                    out.push_str(&format!("archived: {name} · {title}\n"))
                                }
                                None => out.push_str(&format!("archived: {name}\n")),
                            }
                        }
                    }
                }
                if out.is_empty() {
                    Some(format!("(no threads matching \"{query}\")"))
                } else {
                    Some(out.trim_end().to_string())
                }
            }
            "/sessions" => {
                let query = parts.collect::<Vec<_>>().join(" ").to_lowercase();
                let mut paths: Vec<PathBuf> = std::fs::read_dir(store.root().join("sessions"))
                    .ok()
                    .into_iter()
                    .flatten()
                    .flatten()
                    .map(|entry| entry.path())
                    .collect();
                paths.sort();
                let mut lines = Vec::new();
                for path in paths {
                    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                        continue;
                    };
                    let title = first_user_title(&path);
                    let hay = format!(
                        "{} {}",
                        name.to_lowercase(),
                        title.as_deref().unwrap_or("").to_lowercase()
                    );
                    if query.split_whitespace().all(|needle| hay.contains(needle)) {
                        match title {
                            Some(title) => lines.push(format!("{name} · {title}")),
                            None => lines.push(name.to_string()),
                        }
                    }
                }
                if lines.is_empty() {
                    if query.is_empty() {
                        Some("(no sessions)".into())
                    } else {
                        Some(format!("(no sessions matching \"{query}\")"))
                    }
                } else {
                    Some(lines.join("\n"))
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
                let loaded = niucode_core::load(&self.thread).ok()?;
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
            "/cost" => {
                let totals = crate::repl::usage_totals(&load(&self.thread).ok()?.events);
                let mut line = format!(
                    "session: {} turn{} · {} in · {} out",
                    totals.turns,
                    if totals.turns == 1 { "" } else { "s" },
                    crate::render::compact_tokens(totals.input),
                    crate::render::compact_tokens(totals.output)
                );
                if totals.cached > 0 {
                    let pct = totals
                        .cached
                        .saturating_mul(100)
                        .checked_div(totals.input)
                        .unwrap_or(0);
                    line.push_str(&format!(
                        " · {} cached ({pct}%)",
                        crate::render::compact_tokens(totals.cached)
                    ));
                }
                Some(line)
            }
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

fn first_user_title(path: &Path) -> Option<String> {
    let loaded = niucode_core::load(path).ok()?;
    for event in &loaded.events {
        if let Event::UserMessage { text, .. } = event {
            let line = text.lines().next().unwrap_or(text).trim();
            if line.is_empty() {
                continue;
            }
            let title: String = line.chars().take(60).collect();
            return Some(if line.chars().count() > 60 {
                format!("{title}…")
            } else {
                title
            });
        }
    }
    None
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
    fn turn_end_writes_tokens_and_turn_metrics() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        let mut hooks: Box<dyn TurnHooks> = Box::new(CompanionHooks::new(&store).unwrap());
        hooks.on_turn_end(&RunOutcome {
            turns: 1,
            usage: niucode_core::Usage {
                input_tokens: 13_401,
                output_tokens: 5,
                cached_tokens: 13_184,
                reasoning_tokens: 0,
            },
            final_text: "ok".into(),
        });
        let metrics = store.metrics().unwrap();
        assert!(metrics
            .lines()
            .any(|line| line.contains("\ttokens\t13401,5,13184")));
        assert!(metrics.lines().any(|line| line.contains("\tturn\tok")));
    }

    #[test]
    fn new_session_archives_the_old_thread_and_updates_the_pointer() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        std::fs::write(
            store.session_path(),
            "{\"type\":\"session_end\",\"ts\":1}\n",
        )
        .unwrap();
        let mut hooks = CompanionHooks::new(&store).unwrap();
        let new_path = hooks.new_session_path().unwrap().expect("custom path");
        assert_eq!(new_path, store.session_path());
        assert!(!store.session_path().exists());
        assert!(store.threads_dir().read_dir().unwrap().next().is_some());
        hooks.on_session_start(&new_path);
        assert_eq!(hooks.thread, new_path);
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
        let brief = hooks.command("/brief").unwrap();
        assert!(brief.contains("niuse brief"));
        let recalled = hooks.command("/recall rust").unwrap();
        assert!(recalled.contains("Rust projects use cargo test"));
        let added = hooks.command("/goals add ship the recall path").unwrap();
        assert!(added.contains("goal g-"));
        let goals = hooks.command("/goals").unwrap();
        assert!(goals.contains("ship the recall path"));
        let goal_id = store.goals().unwrap()[0].id.clone();
        let closed = hooks.command(&format!("/goals done g-{goal_id}")).unwrap();
        assert!(closed.contains("done"));
        assert!(store.goals().unwrap()[0].done);
        assert_eq!(hooks.command("/loops").unwrap(), "(no open loops)");
        assert_eq!(hooks.command("/journal").unwrap(), "(journal is empty)");
        assert!(!hooks.command("/prompt").unwrap().is_empty());
        assert!(hooks.command("/identity").unwrap().contains("missing"));
        assert!(hooks.command("/user").unwrap().contains("missing"));
        assert!(hooks.command("/threads").unwrap().contains("active:"));
        assert_eq!(hooks.command("/sessions").unwrap(), "(no sessions)");
        assert!(hooks.command("/nope").is_none());
        let commands = hooks.commands();
        for name in [
            "/mem",
            "/loops",
            "/journal",
            "/status",
            "/cost",
            "/export",
            "/threads",
            "/sessions",
            "/prompt",
            "/identity",
            "/user",
            "/brief",
            "/evidence",
            "/report",
            "/recall",
            "/goals",
            "/steps",
            "/verify",
        ] {
            assert!(
                commands.iter().any(|(listed, _)| listed == name),
                "{name} missing from TUI commands"
            );
        }

        std::fs::write(
            store.session_path(),
            concat!(
                "{\"type\":\"user_message\",\"text\":\"one\",\"ts\":1}\n",
                "{\"type\":\"turn_end\",\"usage\":{\"input_tokens\":100,\"output_tokens\":10,\"cached_tokens\":40},\"ts\":2}\n",
                "{\"type\":\"user_message\",\"text\":\"two\",\"ts\":3}\n",
                "{\"type\":\"turn_end\",\"usage\":{\"input_tokens\":50,\"output_tokens\":5},\"ts\":4}\n",
            ),
        )
        .unwrap();
        let cost = hooks.command("/cost").unwrap();
        assert!(cost.contains("2 turns"));
        let evidence = hooks.command("/evidence").unwrap();
        assert!(evidence.contains("evidence"));
        let report = hooks.command("/report").unwrap();
        assert!(report.contains("weekly report"), "{report}");
        assert!(cost.contains("150 in"));
        assert!(cost.contains("40 cached (26%)"), "{cost}");

        let added = hooks.command("/steps add write the tests").unwrap();
        assert!(added.contains("p-1"));
        let steps = hooks.command("/steps").unwrap();
        assert!(steps.contains("p-1 write the tests"));
        assert!(steps.contains("no recorded verification"));
        let refused = hooks.command("/steps done 1").unwrap();
        assert!(refused.contains("no passing verification"), "{refused}");
        let failed = hooks.command("/verify p-1 false").unwrap();
        assert!(failed.contains("FAILED"), "{failed}");
        let steps = hooks.command("/steps").unwrap();
        assert!(steps.contains("verification failed"), "{steps}");
        let refused = hooks.command("/steps done 1").unwrap();
        assert!(refused.contains("no passing verification"), "{refused}");
        let verified = hooks.command("/verify p-1 true").unwrap();
        assert!(verified.contains("passed"), "{verified}");
        let closed = hooks.command("/steps done 1").unwrap();
        assert!(closed.contains("done p-1"), "{closed}");
        assert!(store.plan().unwrap()[0].done);
    }

    #[test]
    fn memory_off_suppresses_candidates_but_keeps_journal() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        let mut hooks = CompanionHooks::with_memory(&store, false, store.session_path()).unwrap();
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
            attachments: Vec::new(),
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
        std::fs::write(
            store.threads_dir().join("thread-1.jsonl"),
            "{\"type\":\"user_message\",\"text\":\"fix rust clippy\",\"ts\":1}\n",
        )
        .unwrap();
        std::fs::write(
            store.threads_dir().join("thread-2.jsonl"),
            "{\"type\":\"user_message\",\"text\":\"draft calendar\",\"ts\":2}\n",
        )
        .unwrap();
        std::fs::create_dir_all(store.root().join("sessions")).unwrap();
        std::fs::write(
            store.root().join("sessions").join("s-1.jsonl"),
            "{\"type\":\"user_message\",\"text\":\"audit budget\",\"ts\":1}\n",
        )
        .unwrap();
        std::fs::write(
            store.root().join("sessions").join("s-2.jsonl"),
            "{\"type\":\"user_message\",\"text\":\"smoke check\",\"ts\":2}\n",
        )
        .unwrap();
        let mut hooks = CompanionHooks::new(&store).unwrap();
        assert!(hooks
            .command("/threads")
            .unwrap()
            .contains("thread-1.jsonl"));
        let filtered_threads = hooks.command("/threads calendar").unwrap();
        assert!(filtered_threads.contains("thread-2.jsonl"));
        assert!(!filtered_threads.contains("thread-1.jsonl"));

        assert!(hooks.command("/sessions").unwrap().contains("s-1.jsonl"));
        let filtered_sessions = hooks.command("/sessions budget").unwrap();
        assert!(filtered_sessions.contains("s-1.jsonl"));
        assert!(!filtered_sessions.contains("s-2.jsonl"));
    }

    #[test]
    fn multiple_candidates_keep_their_indices() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        let mut hooks = CompanionHooks::new(&store).unwrap();
        let mut text = "<<niuse:memory kind=preference tags=rust>>\nfirst\n<</niuse:memory>>\n\
            <<niuse:memory kind=lesson tags=rust>>\nsecond\n<</niuse:memory>>"
            .to_string();
        hooks.filter_text(&mut text);
        let prompts = hooks.after_turn(&outcome());
        assert_eq!(prompts.len(), 2);
        hooks.on_answer(&prompts[1].id, 0);
        hooks.on_answer(&prompts[0].id, 0);
        let memories = store.memories().unwrap();
        assert_eq!(memories.len(), 2);
        let texts: Vec<&str> = memories.iter().map(|entry| entry.text.as_str()).collect();
        assert!(texts.contains(&"first"));
        assert!(texts.contains(&"second"));
    }

    #[test]
    fn custom_thread_is_used_by_listing_and_export() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        store.init().unwrap();
        let custom = directory.path().join("custom-thread.jsonl");
        let log = niucode_core::SessionLog::create(directory.path()).unwrap();
        log.append(&Event::UserMessage {
            text: "custom prompt".into(),
            attachments: Vec::new(),
            ts: 1,
        })
        .unwrap();
        let source = log.path().to_path_buf();
        drop(log);
        std::fs::rename(source, &custom).unwrap();

        let mut hooks = CompanionHooks::new_with_thread(&store, custom.clone()).unwrap();
        assert!(hooks
            .command("/threads")
            .unwrap()
            .contains("custom-thread.jsonl"));
        let target = directory.path().join("custom.md");
        hooks
            .command(&format!("/export {}", target.display()))
            .unwrap();
        assert!(std::fs::read_to_string(target)
            .unwrap()
            .contains("custom prompt"));
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
        // New-memory options are accept, edit, discard; index 1 opens $EDITOR.
        hooks.on_answer(&prompts[0].id, 2);
        assert!(store.memories().unwrap().is_empty());
    }
}
