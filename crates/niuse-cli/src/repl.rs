//! Interactive REPL and one-shot execution.

use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use niucode_agent::ApprovalMode;
use niucode_core::load;
use niucode_provider::Provider;
use niucode_tools::SandboxPolicy;
use niuse_core::policy::{conflicts, secret_reason};
use niuse_core::prompt::compose;
use niuse_core::{Block, Date, LoopItem, MemoryEntry, MemoryKind, Store};
use rustyline::completion::{Completer, Pair};
use rustyline::config::Config;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::FileHistory;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context as RlContext, Editor, Helper};

const SLASH_COMMANDS: &[&str] = &[
    "/help",
    "/quit",
    "/exit",
    "/mem",
    "/mem recall",
    "/mem show",
    "/mem review",
    "/remember",
    "/forget",
    "/loops",
    "/loop",
    "/done",
    "/journal",
    "/journal search",
    "/journal add",
    "/stats",
    "/doctor",
    "/identity",
    "/identity edit",
    "/user",
    "/user edit",
    "/export",
    "/new",
    "/cd",
    "/status",
    "/prompt",
    "/model",
];

fn slash_candidates(prefix: &str) -> Vec<&'static str> {
    if !prefix.starts_with('/') || prefix.ends_with(' ') {
        return Vec::new();
    }
    let mut matches: Vec<&'static str> = SLASH_COMMANDS
        .iter()
        .copied()
        .filter(|command| command.starts_with(prefix))
        .collect();
    matches.sort_by_key(|command| command.len());
    matches
}

#[derive(Clone, Copy)]
enum IdScope {
    Memory,
    Loop,
}

/// Where an id completion starts, the fragment typed so far, and which store.
fn id_query(line: &str) -> Option<(usize, &str, IdScope)> {
    if let Some(rest) = line.strip_prefix("/mem ") {
        let (sub, fragment) = rest.split_once(' ')?;
        let scope = match sub {
            "show" | "edit" | "confirm" | "forget" => IdScope::Memory,
            _ => return None,
        };
        return Some((line.len() - fragment.len(), fragment, scope));
    }
    for (command, scope) in [("/forget ", IdScope::Memory), ("/done ", IdScope::Loop)] {
        if let Some(fragment) = line.strip_prefix(command) {
            if !fragment.contains(' ') {
                return Some((command.len(), fragment, scope));
            }
        }
    }
    None
}

fn id_candidates(store: &Store, scope: IdScope, fragment: &str) -> Vec<Pair> {
    let items: Vec<(String, String)> = match scope {
        IdScope::Memory => store
            .memories()
            .unwrap_or_default()
            .into_iter()
            .map(|entry| (entry.id, entry.text))
            .collect(),
        IdScope::Loop => store
            .loops()
            .unwrap_or_default()
            .into_iter()
            .filter(|item| !item.done)
            .map(|item| (format!("l-{}", item.id), item.text))
            .collect(),
    };
    let mut matches: Vec<Pair> = items
        .into_iter()
        .filter(|(id, _)| id.starts_with(fragment))
        .map(|(id, text)| Pair {
            display: format!("{id}  {}", text.chars().take(50).collect::<String>()),
            replacement: id,
        })
        .collect();
    matches.sort_by(|left, right| left.replacement.cmp(&right.replacement));
    matches
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathScope {
    Directory,
    Any,
}

fn path_query(line: &str) -> Option<(usize, &str, PathScope)> {
    if let Some(fragment) = line.strip_prefix("/cd ") {
        return Some(("/cd ".len(), fragment, PathScope::Directory));
    }
    if let Some(fragment) = line.strip_prefix("/export ") {
        return Some(("/export ".len(), fragment, PathScope::Any));
    }
    None
}

fn path_candidates(fragment: &str, cwd: &Path, scope: PathScope) -> Vec<Pair> {
    let (dir_prefix, name_prefix) = match fragment.rfind('/') {
        Some(index) => (&fragment[..=index], &fragment[index + 1..]),
        None => ("", fragment),
    };
    let base = if let Some(rest) = dir_prefix.strip_prefix("~/") {
        match std::env::var_os("HOME") {
            Some(home) => PathBuf::from(home).join(rest),
            None => return Vec::new(),
        }
    } else if dir_prefix.starts_with('/') {
        PathBuf::from(dir_prefix)
    } else if dir_prefix.is_empty() {
        cwd.to_path_buf()
    } else {
        cwd.join(dir_prefix)
    };

    let entries = match fs::read_dir(&base) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };
    let mut matches = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with(name_prefix) {
            continue;
        }
        if name.starts_with('.') && !name_prefix.starts_with('.') {
            continue;
        }
        let is_dir = entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false);
        if scope == PathScope::Directory && !is_dir {
            continue;
        }
        let mut replacement = format!("{dir_prefix}{name}");
        if is_dir {
            replacement.push('/');
        }
        matches.push(Pair {
            display: replacement.clone(),
            replacement,
        });
    }
    matches.sort_by(|left, right| left.replacement.cmp(&right.replacement));
    matches
}

struct SlashCompleter {
    root: PathBuf,
    cwd: Arc<Mutex<PathBuf>>,
}

impl Completer for SlashCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &RlContext<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let typed = &line[..pos];
        if let Some((start, fragment, scope)) = path_query(typed) {
            let cwd = self
                .cwd
                .lock()
                .map(|guard| guard.clone())
                .unwrap_or_else(|poisoned| poisoned.into_inner().clone());
            return Ok((start, path_candidates(fragment, &cwd, scope)));
        }
        if let Some((start, fragment, scope)) = id_query(typed) {
            let store = Store::at(self.root.clone());
            return Ok((start, id_candidates(&store, scope, fragment)));
        }
        let candidates = slash_candidates(typed)
            .into_iter()
            .map(|command| Pair {
                display: command.to_string(),
                replacement: format!("{command} "),
            })
            .collect();
        Ok((0, candidates))
    }
}

impl Hinter for SlashCompleter {
    type Hint = String;
}

impl Highlighter for SlashCompleter {}

impl Validator for SlashCompleter {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        if ctx
            .input()
            .lines()
            .last()
            .is_some_and(|line| line.ends_with('\\'))
        {
            Ok(ValidationResult::Incomplete)
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}

impl Helper for SlashCompleter {}

use crate::commands;
use crate::config::{build_provider, resolve_cwd, resolve_policy};
use crate::driver::{self, SessionMeta, Thread};
use crate::render::{self, Renderer};
use crate::settings::Settings;
use crate::Cli;

pub struct App {
    pub store: Store,
    pub provider: Arc<dyn Provider>,
    pub provider_id: String,
    pub model: String,
    no_thinking: bool,
    pub policy: SandboxPolicy,
    pub approval: ApprovalMode,
    pub allow_unsandboxed: bool,
    pub cwd: PathBuf,
    completion_cwd: Arc<Mutex<PathBuf>>,
    pub max_steps: usize,
    pub prompt: String,
    pub prompt_tokens: usize,
    pub thread_path: PathBuf,
    memory_enabled: bool,
    suppress_candidates: bool,
}

impl App {
    pub fn new(cli: &Cli, settings: &Settings, store: Store) -> Result<Self> {
        let bundle = build_provider(
            &settings.provider,
            settings.model.as_deref(),
            cli.no_thinking,
        )?;
        let (policy, approval, allow_unsandboxed) =
            resolve_policy(&settings.sandbox, &settings.approval, &settings.network)?;
        let cwd = resolve_cwd(cli.cwd.clone())?;
        let completion_cwd = Arc::new(Mutex::new(cwd.clone()));
        let memory_enabled = std::env::var("NIUSE_MEMORY")
            .map(|value| value.trim().to_lowercase() != "off")
            .unwrap_or(true);

        let thread_path = cli.thread.clone().unwrap_or_else(|| store.session_path());

        let mut app = Self {
            store,
            provider: bundle.provider,
            provider_id: bundle.provider_id,
            model: bundle.model,
            no_thinking: cli.no_thinking,
            policy,
            approval,
            allow_unsandboxed,
            cwd,
            completion_cwd,
            max_steps: settings.max_steps,
            prompt: String::new(),
            prompt_tokens: 0,
            thread_path,
            memory_enabled,
            suppress_candidates: false,
        };
        app.rebuild_prompt()?;
        Ok(app)
    }

    pub fn rebuild_prompt(&mut self) -> Result<()> {
        let today = Date::today();
        let identity = self.store.identity()?;
        let user = self.store.user()?;
        let memories = if self.memory_enabled {
            self.store.memories()?
        } else {
            Vec::new()
        };
        let loops = self.store.loops()?;
        let journal = self.store.recent_journal(3)?;
        let composition = compose(
            identity.as_deref(),
            user.as_deref(),
            &memories,
            &loops,
            &journal,
            today,
        );
        self.prompt = composition.system;
        self.prompt_tokens = composition.tokens;
        Ok(())
    }

    pub async fn run(mut self) -> Result<()> {
        let path = self.thread_path.clone();
        let meta = SessionMeta {
            cwd: &self.cwd,
            model: &self.model,
            provider: &self.provider_id,
            policy: self.policy,
            approval: self.approval,
        };
        let mut thread = driver::open_thread(&path, &meta)?;
        let mut history = driver::history(&thread.path)?;
        self.banner();

        let history_path = self.store.history_path();
        let mut editor = if io::stdin().is_terminal() {
            match Editor::<SlashCompleter, FileHistory>::with_config(Config::default()) {
                Ok(mut editor) => {
                    editor.set_helper(Some(SlashCompleter {
                        root: self.store.root().to_path_buf(),
                        cwd: self.completion_cwd.clone(),
                    }));
                    if history_path.exists() {
                        if let Err(error) = editor.load_history(&history_path) {
                            eprintln!("warning: cannot load history: {error}");
                        }
                    }
                    Some(editor)
                }
                Err(error) => {
                    eprintln!("warning: line editing unavailable: {error}");
                    None
                }
            }
        } else {
            None
        };

        loop {
            let prompt = format!("\n{}", self.prompt_label());
            let line = match editor.as_mut() {
                Some(editor) => match editor.readline(&prompt) {
                    Ok(line) => {
                        let line = join_continuations(&line);
                        let _ = editor.add_history_entry(line.as_str());
                        line
                    }
                    Err(ReadlineError::Interrupted) => {
                        println!("^C");
                        continue;
                    }
                    Err(ReadlineError::Eof) => {
                        println!();
                        break;
                    }
                    Err(error) => {
                        eprintln!("input error: {error}");
                        break;
                    }
                },
                None => {
                    print!("{prompt}");
                    let _ = io::stdout().flush();
                    let Some(line) = render::read_line() else {
                        println!();
                        break;
                    };
                    line
                }
            };
            let input = line.trim();
            if input.is_empty() {
                continue;
            }
            if input == "/new" {
                driver::append_end(&thread.log)?;
                if let Some(archived) =
                    driver::archive_thread(&thread.path, &self.store.threads_dir())?
                {
                    println!("archived {}", archived.display());
                }
                thread = {
                    let meta = SessionMeta {
                        cwd: &self.cwd,
                        model: &self.model,
                        provider: &self.provider_id,
                        policy: self.policy,
                        approval: self.approval,
                    };
                    driver::open_thread(&self.store.session_path(), &meta)?
                };
                history.clear();
                self.rebuild_prompt()?;
                println!("new thread {}", thread.path.display());
                continue;
            }
            if input.starts_with('/') {
                if matches!(self.command(input, &thread)?, Flow::Quit) {
                    break;
                }
                continue;
            }

            let config = driver::agent_config(
                &self.provider_id,
                &self.model,
                &self.cwd,
                self.prompt.clone(),
                self.policy,
                self.approval,
                self.allow_unsandboxed,
                self.max_steps,
            );
            let mut renderer = Renderer::new();
            let result = driver::run_turn(
                self.provider.as_ref(),
                &mut thread.log,
                &config,
                input,
                history,
                &mut renderer,
            )
            .await;
            let parsed = renderer.finish();
            match result {
                Ok(Some(_)) => {
                    self.metric("turn", "ok");
                    let source = thread.log.id().to_string();
                    self.handle_blocks(parsed.blocks, &source)?;
                }
                Ok(None) => {
                    self.metric("turn", "cancelled");
                    println!("[cancelled]");
                }
                Err(error) => {
                    self.metric("turn", "error");
                    eprintln!("error: {error:#}");
                }
            }
            history = match driver::history(&thread.path) {
                Ok(history) => history,
                Err(error) => {
                    eprintln!("warning: cannot reload session history: {error:#}");
                    Vec::new()
                }
            };
        }

        driver::append_end(&thread.log)?;
        if let Some(mut editor) = editor {
            if let Err(error) = editor.save_history(&history_path) {
                eprintln!("warning: cannot save history: {error}");
            }
            secure_history(&history_path);
        }
        Ok(())
    }

    fn banner(&self) {
        let today = Date::today();
        let memories = self.store.memories().unwrap_or_default();
        let stale = memories
            .iter()
            .filter(|entry| entry.is_stale(today) && !entry.is_expired(today))
            .count();
        let open = self
            .store
            .loops()
            .unwrap_or_default()
            .iter()
            .filter(|item| !item.done)
            .count();

        println!(
            "niuse · {} · {} / {}",
            self.model,
            self.policy.mode.as_str(),
            self.approval.as_str()
        );
        println!(
            "{} 条记忆（{} 条待验证）· {} 条未完成承诺 · prompt ~{} tokens",
            memories.len(),
            stale,
            open,
            self.prompt_tokens
        );
        println!("/help 查看命令 · Ctrl-C 取消当前 turn · Ctrl-D 退出");
        if !self.memory_enabled {
            println!("NIUSE_MEMORY=off：本次 session 不注入记忆，也不提议新记忆。");
        }
        if self.store.identity().unwrap_or(None).is_none() {
            println!("提示：还没有 identity.md，运行 `niuse identity init` 定义它是谁。");
        }
    }

    fn prompt_label(&self) -> String {
        let name = self
            .cwd
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| self.cwd.display().to_string());
        format!("{name} › ")
    }

    fn command(&mut self, input: &str, thread: &Thread) -> Result<Flow> {
        let (command, argument) = input.split_once(' ').unwrap_or((input, ""));
        let argument = argument.trim();

        match command {
            "/help" => print_help(),
            "/quit" | "/exit" => return Ok(Flow::Quit),
            "/mem" => {
                if argument == "review" {
                    commands::mem_review(&self.store)?;
                } else if let Some(query) = argument.strip_prefix("recall ") {
                    commands::mem_search(&self.store, query.trim(), false, false, None)?;
                } else if let Some(id) = argument.strip_prefix("show ") {
                    commands::mem_show(&self.store, id.trim())?;
                } else {
                    commands::mem_list(&self.store, false, false, None)?;
                }
            }
            "/remember" => {
                if argument.is_empty() {
                    println!("用法：/remember <一句话>");
                } else if let Some(reason) = secret_reason(argument) {
                    println!("拒绝：疑似包含 {reason}。");
                } else {
                    let entry = self.store.add_memory(MemoryEntry::new(
                        MemoryKind::Preference,
                        argument,
                        Vec::new(),
                        Date::today(),
                        Some(thread.log.id().to_string()),
                    ))?;
                    self.metric("remember", &entry.id);
                    println!("已记住 m-{}（下一个 session 生效）", entry.id);
                }
            }
            "/forget" => {
                if argument.is_empty() {
                    println!("用法：/forget <id>");
                } else {
                    commands::mem_forget(&self.store, argument)?;
                    self.metric("forget", argument);
                }
            }
            "/export" => {
                let path = export_path(&self.store, &thread.path, argument, &self.cwd);
                commands::export(&self.store, Some(&path), Some(&thread.path))?;
            }
            "/journal" => {
                if let Some(query) = argument.strip_prefix("search ") {
                    commands::journal_search(&self.store, query.trim(), false)?;
                } else if let Some(text) = argument.strip_prefix("add ") {
                    commands::journal_add(&self.store, text.trim())?;
                } else {
                    commands::journal_show(&self.store, 7, false)?;
                }
            }
            "/stats" => commands::stats(&self.store, 7, false, false)?,
            "/doctor" => {
                if let Err(error) = commands::doctor(&self.store, &self.provider_id, false) {
                    println!("{error}");
                }
            }
            "/loops" => commands::loops_list(&self.store, false, false)?,
            "/loop" => {
                if argument.is_empty() {
                    println!("用法：/loop <一件你答应要做的事>");
                } else if let Some(reason) = secret_reason(argument) {
                    println!("拒绝：疑似包含 {reason}。");
                } else {
                    let item = self.store.add_loop(LoopItem::new(argument, None))?;
                    self.metric("loop", &item.id);
                    println!("承诺已记录 l-{}", item.id);
                }
            }
            "/done" => {
                if argument.is_empty() {
                    println!("用法：/done <l-id>");
                } else {
                    commands::loops_done(&self.store, argument)?;
                }
            }
            "/cd" => {
                if argument.is_empty() {
                    println!("cwd: {}", self.cwd.display());
                } else {
                    let path = PathBuf::from(argument);
                    let path = if path.is_absolute() {
                        path
                    } else {
                        self.cwd.join(path)
                    };
                    let path = path
                        .canonicalize()
                        .with_context(|| format!("resolve {}", path.display()))?;
                    if !path.is_dir() {
                        bail!("{} is not a directory", path.display());
                    }
                    self.cwd = path;
                    if let Ok(mut guard) = self.completion_cwd.lock() {
                        *guard = self.cwd.clone();
                    }
                    println!("cwd: {}", self.cwd.display());
                }
            }
            "/model" => {
                if argument.is_empty() {
                    println!("model: {} · provider {}", self.model, self.provider_id);
                } else {
                    match build_provider(
                        &self.provider_id,
                        model_override(argument),
                        self.no_thinking,
                    ) {
                        Ok(bundle) => {
                            self.provider = bundle.provider;
                            self.model = bundle.model;
                            println!("model: {}", self.model);
                        }
                        Err(error) => eprintln!("cannot switch model: {error:#}"),
                    }
                }
            }
            "/status" => {
                commands::status(&self.store, &thread.path, false)?;
                let events = load(&thread.path)
                    .map(|loaded| loaded.events.len())
                    .unwrap_or(0);
                println!("provider  {} / {}", self.provider_id, self.model);
                println!(
                    "sandbox   {} / approval {} / network {}",
                    self.policy.mode.as_str(),
                    self.approval.as_str(),
                    self.policy.network.as_str()
                );
                println!("cwd       {}", self.cwd.display());
                println!("session   {} events", events);
            }
            "/user" => {
                if argument == "edit" {
                    commands::user_edit(&self.store)?;
                } else {
                    commands::user_show(&self.store)?;
                }
            }
            "/identity" => {
                if argument == "edit" {
                    commands::identity_edit(&self.store)?;
                } else {
                    commands::identity_show(&self.store)?;
                }
            }
            "/prompt" => println!("{}", self.prompt),
            other => println!("未知命令 {other}；/help 查看可用命令。"),
        }
        Ok(Flow::Continue)
    }

    fn handle_blocks(&mut self, blocks: Vec<Block>, source: &str) -> Result<()> {
        let today = Date::today();
        if !self.memory_enabled {
            // Journal and loops still record the session; memory candidates are
            // suppressed so an A/B run cannot change the store.
            for block in blocks {
                match block {
                    Block::Journal { bullets } => {
                        if self.store.append_journal(today, &bullets)? {
                            self.metric("journal", &bullets.len().to_string());
                        }
                    }
                    Block::Loop { due, text } => {
                        let item = self.store.add_loop(LoopItem::new(text, due))?;
                        self.metric("loop", &item.id);
                    }
                    Block::Memory { .. } => {}
                }
            }
            return Ok(());
        }
        let mut offered_memory = false;

        for block in blocks {
            match block {
                Block::Journal { bullets } => {
                    if self.store.append_journal(today, &bullets)? {
                        self.metric("journal", &bullets.len().to_string());
                        println!("journal +{}", bullets.len());
                    }
                }
                Block::Loop { due, text } => {
                    let item = self.store.add_loop(LoopItem::new(text, due))?;
                    self.metric("loop", &item.id);
                    println!("承诺已记录 l-{}", item.id);
                }
                Block::Memory { kind, tags, text } => {
                    if offered_memory || self.suppress_candidates {
                        continue;
                    }
                    offered_memory = true;
                    self.offer_memory(kind, tags, text, source)?;
                }
            }
        }
        Ok(())
    }

    fn offer_memory(
        &mut self,
        kind: MemoryKind,
        tags: Vec<String>,
        text: String,
        source: &str,
    ) -> Result<()> {
        if let Some(reason) = secret_reason(&text) {
            self.metric("blocked", reason);
            println!("\n[记忆候选被拒绝] 疑似包含 {reason}，没有写入。");
            return Ok(());
        }

        let existing = self.store.memories()?;
        let conflicts: Vec<&MemoryEntry> = existing
            .iter()
            .filter(|entry| conflicts(&entry.text, &entry.tags, &text, &tags))
            .collect();

        let tag_label = if tags.is_empty() {
            String::new()
        } else {
            format!(" · {}", tags.join(", "))
        };
        println!("\n记忆候选 [{}{tag_label}]", kind.as_str());
        println!("  {text}");
        self.metric("propose", &tags.join(","));

        if conflicts.is_empty() {
            print!("[y] 接受  [n] 丢弃  [e] 编辑  [s] 本次 session 不再提议 > ");
            let _ = io::stdout().flush();
            match render::read_line()
                .unwrap_or_default()
                .trim()
                .to_lowercase()
                .as_str()
            {
                "y" | "yes" => {
                    let entry = self.save_memory(kind, text, tags, source)?;
                    self.metric("accept", &entry.id);
                    println!("已记住 m-{}", entry.id);
                }
                "e" | "edit" => {
                    if let Some(edited) = commands::edit_text(&text) {
                        if let Some(reason) = secret_reason(&edited) {
                            println!("拒绝：疑似包含 {reason}。");
                        } else {
                            let entry = self.save_memory(kind, edited, tags, source)?;
                            self.metric("accept", &entry.id);
                            self.metric("edit", &entry.id);
                            println!("已记住 m-{}", entry.id);
                        }
                    } else {
                        println!("未修改，已丢弃。");
                    }
                }
                "s" | "stop" => {
                    self.suppress_candidates = true;
                    self.metric("suppress", "");
                    println!("本次 session 不再提议。");
                }
                _ => {
                    self.metric("discard", "");
                    println!("已丢弃。");
                }
            }
            return Ok(());
        }

        for old in &conflicts {
            println!("  旧 m-{}：{}", old.id, old.text);
        }
        print!("[r] 替换  [k] 并存  [d] 丢弃 > ");
        let _ = io::stdout().flush();
        match render::read_line()
            .unwrap_or_default()
            .trim()
            .to_lowercase()
            .as_str()
        {
            "r" | "replace" => {
                if let Some(old) = conflicts.first() {
                    self.store.remove_memory(&old.id)?;
                }
                let entry = self.save_memory(kind, text, tags, source)?;
                self.metric("accept", &format!("replace:{}", entry.id));
                println!("已替换，新 id m-{}", entry.id);
            }
            "k" | "keep" => {
                let entry = self.save_memory(kind, text, tags, source)?;
                self.metric("accept", &format!("keep:{}", entry.id));
                println!("已并存，新 id m-{}", entry.id);
            }
            _ => {
                self.metric("discard", "conflict");
                println!("已丢弃。");
            }
        }
        Ok(())
    }

    fn metric(&self, kind: &str, detail: &str) {
        if let Err(error) = self.store.append_metric(kind, detail) {
            eprintln!("warning: cannot record metric: {error:#}");
        }
    }

    fn save_memory(
        &self,
        kind: MemoryKind,
        text: String,
        tags: Vec<String>,
        source: &str,
    ) -> Result<MemoryEntry> {
        let entry = MemoryEntry::new(kind, text, tags, Date::today(), Some(source.to_string()));
        self.store.add_memory(entry)
    }
}

enum Flow {
    Continue,
    Quit,
}

pub async fn run_exec(app: App, prompt: String) -> Result<()> {
    let log = driver::new_session(&app.store.root().join("sessions"))?;
    let meta = SessionMeta {
        cwd: &app.cwd,
        model: &app.model,
        provider: &app.provider_id,
        policy: app.policy,
        approval: app.approval,
    };
    driver::append_start(&log, &meta)?;

    let config = driver::agent_config(
        &app.provider_id,
        &app.model,
        &app.cwd,
        app.prompt.clone(),
        app.policy,
        app.approval,
        app.allow_unsandboxed,
        app.max_steps,
    );
    let mut renderer = Renderer::new();
    let mut log = log;
    let result = driver::run_turn(
        app.provider.as_ref(),
        &mut log,
        &config,
        &prompt,
        Vec::new(),
        &mut renderer,
    )
    .await;
    let parsed = renderer.finish();

    for block in parsed.blocks {
        match block {
            Block::Memory { kind, tags, text } => {
                eprintln!("[candidate:{}:{}] {text}", kind.as_str(), tags.join(","));
            }
            Block::Loop { due, text } => {
                eprintln!(
                    "[loop] {text}{}",
                    due.map(|d| format!(" ({d})")).unwrap_or_default()
                );
            }
            Block::Journal { bullets } => {
                for bullet in bullets {
                    eprintln!("[journal] {bullet}");
                }
            }
        }
    }

    let status = match &result {
        Ok(Some(_)) => "ok",
        Ok(None) => "cancelled",
        Err(_) => "error",
    };
    app.metric("turn", status);

    driver::append_end(&log)?;
    println!("\nsession: {}", log.path().display());
    result.map(|_| ())
}

fn model_override(argument: &str) -> Option<&str> {
    if argument.eq_ignore_ascii_case("default") {
        None
    } else {
        Some(argument)
    }
}

fn join_continuations(input: &str) -> String {
    input
        .split('\n')
        .map(|line| line.strip_suffix('\\').unwrap_or(line).trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn export_path(store: &Store, thread: &Path, argument: &str, cwd: &Path) -> PathBuf {
    if !argument.is_empty() {
        let path = PathBuf::from(argument);
        if path.is_absolute() {
            return path;
        }
        return cwd.join(path);
    }
    let stem = thread
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "thread".to_string());
    store.root().join(format!("export-{stem}.md"))
}

#[cfg(unix)]
fn secure_history(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = std::fs::metadata(path) {
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        let _ = std::fs::set_permissions(path, permissions);
    }
}

#[cfg(not(unix))]
fn secure_history(_path: &Path) {}

fn print_help() {
    println!(
        "对话：直接输入；空行忽略。\n\
         /mem            列出现行记忆\n\
         /mem recall <query>  检索记忆，包含已过期条目\n\
         /mem show <id>  查看单条记忆的全部字段\n\
         /mem review     复核过期条目（y 仍然成立 / d 删除 / s 跳过）\n\
         /remember <一句话>  直接记住一条偏好\n\
         /forget <id>    删除一条记忆\n\
         /loops          列出承诺\n\
         /loop <text>    记下一条承诺\n\
         /done <l-id>    关闭一条承诺\n\
         /journal        最近 7 天\n\
         /journal search <query>  检索日记条目\n\
         /journal add <text>      追加一条今天的日记\n\
        /export [path]  导出当前主线为 Markdown\n\
        （终端里行尾加 \\ 可续行）\n\
         /stats          最近 7 天的候选接受率与活动计数\n\
         /doctor         检查本地设置，不调用模型\n\
         /identity       查看 identity.md（/identity edit 编辑）\n\
         /user           查看 user.md（/user edit 编辑）\n\
         /export [path]  导出当前主线为 Markdown\n\
        （终端里行尾加 \\ 可续行）\n\
        /new            归档当前主线并开始一条新的\n\
         /cd <path>      切换工作目录（下一个 turn 生效）\n\
         /model [name]   查看或切换模型（default 回退）\n\
        /status         当前配置与存储状态\n\
         /prompt         打印完整 system prompt（调试用）\n\
         /help           本帮助\n\
         /quit           退出"
    );
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn slash_completion_matches_prefixes() {
        assert_eq!(slash_candidates("/mem rec"), vec!["/mem recall"]);
        assert!(slash_candidates("/mem r").contains(&"/mem review"));
        assert!(slash_candidates("/mem").contains(&"/mem review"));
        assert_eq!(slash_candidates("/zzz"), Vec::<&str>::new());
        assert_eq!(slash_candidates("hello"), Vec::<&str>::new());
        assert_eq!(slash_candidates("/mem "), Vec::<&str>::new());
    }

    #[test]
    fn default_model_means_provider_default() {
        assert_eq!(model_override("default"), None);
        assert_eq!(model_override("DEFAULT"), None);
        assert_eq!(model_override("deepseek-chat"), Some("deepseek-chat"));
    }

    #[test]
    fn backslash_continuations_join_lines() {
        assert_eq!(join_continuations("hello \\\nthere"), "hello\nthere");
        assert_eq!(join_continuations("one"), "one");
        assert_eq!(join_continuations("a \\\nb \\\nc"), "a\nb\nc");
    }

    #[test]
    fn path_completion_lists_directories_and_files() {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir(directory.path().join("alpha")).unwrap();
        fs::write(directory.path().join("beta.txt"), "").unwrap();

        let all = path_candidates("", directory.path(), PathScope::Any);
        let names: Vec<&str> = all.iter().map(|pair| pair.replacement.as_str()).collect();
        assert!(names.contains(&"alpha/"));
        assert!(names.contains(&"beta.txt"));

        let dirs = path_candidates("", directory.path(), PathScope::Directory);
        assert!(!dirs.is_empty());
        assert!(dirs.iter().all(|pair| pair.replacement.ends_with('/')));

        let filtered = path_candidates("al", directory.path(), PathScope::Any);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].replacement, "alpha/");

        assert_eq!(path_query("/cd sr"), Some((4, "sr", PathScope::Directory)));
        assert_eq!(path_query("/export out"), Some((8, "out", PathScope::Any)));
    }

    #[test]
    fn export_path_defaults_next_to_the_home() {
        let home = Path::new("/tmp/niuse");
        let thread = Path::new("/tmp/niuse/threads/thread-42.jsonl");
        assert_eq!(
            export_path(
                &Store::at(home.to_path_buf()),
                thread,
                "",
                Path::new("/tmp/niuse")
            ),
            PathBuf::from("/tmp/niuse/export-thread-42.md")
        );
        assert_eq!(
            export_path(
                &Store::at(home.to_path_buf()),
                thread,
                "out.md",
                Path::new("/work")
            ),
            PathBuf::from("/work/out.md")
        );
    }

    #[test]
    fn id_completion_reads_the_stores() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().join(".niuse"));
        let first = store
            .add_memory(MemoryEntry::new(
                MemoryKind::Preference,
                "prefer short paragraphs",
                vec!["style".into()],
                Date::today(),
                None,
            ))
            .unwrap();
        store
            .add_memory(MemoryEntry::new(
                MemoryKind::Lesson,
                "use nextest",
                vec!["rust".into()],
                Date::today(),
                None,
            ))
            .unwrap();
        let matches = id_candidates(&store, IdScope::Memory, &first.id);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].replacement, first.id);

        let line = format!("/mem show {}", &first.id[..2]);
        let (start, fragment, _) = id_query(&line).unwrap();
        assert_eq!(start, "/mem show ".len());
        assert_eq!(fragment, &first.id[..2]);
    }

    #[test]
    fn history_file_is_private() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("history");
        std::fs::write(&path, "/help\n").unwrap();
        secure_history(&path);
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
