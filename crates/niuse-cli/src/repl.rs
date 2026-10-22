//! Interactive REPL and one-shot execution.

use std::collections::HashSet;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use niucode_agent::ApprovalMode;
use niucode_core::{load, Event};
use niucode_mcp::McpHost;
use niucode_provider::Provider;
use niucode_tools::SandboxPolicy;
use niuse_core::policy::{conflicts, secret_reason};
use niuse_core::prompt::compose_with_plan;
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
    "/recall",
    "/remember",
    "/forget",
    "/loops",
    "/loop",
    "/done",
    "/goals",
    "/goals add",
    "/goals done",
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
    "/threads",
    "/sessions",
    "/resume",
    "/new",
    "/cd",
    "/status",
    "/evidence",
    "/report",
    "/verify",
    "/brief",
    "/plan",
    "/prompt",
    "/model",
    "/cost",
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
    let (prefix, fragment) = match scope {
        IdScope::Memory => match fragment.strip_prefix("m-") {
            Some(rest) => ("m-", rest),
            None => ("", fragment),
        },
        IdScope::Loop => ("", fragment),
    };
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
            display: format!(
                "{prefix}{id}  {}",
                text.chars().take(50).collect::<String>()
            ),
            replacement: format!("{prefix}{id}"),
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
    if fragment == "~" || fragment == "." || fragment == ".." {
        let completed = format!("{fragment}/");
        return vec![Pair {
            display: completed.clone(),
            replacement: completed,
        }];
    }
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
use crate::config::{build_provider, resolve_cwd, resolve_policy, resolve_thread};
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
    pub plan_mode: bool,
    pub mcp_host: Option<Arc<McpHost>>,
    memory_enabled: bool,
    suppress_candidates: bool,
    rejected_tags: HashSet<String>,
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

        let thread_path = resolve_thread(cli.thread.clone(), &store)?;

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
            plan_mode: cli.plan,
            mcp_host: None,
            memory_enabled,
            suppress_candidates: false,
            rejected_tags: HashSet::new(),
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
        let goals = self.store.goals()?;
        let plan = self.store.plan()?;
        let journal = self.store.recent_journal(3)?;
        let composition = compose_with_plan(
            identity.as_deref(),
            user.as_deref(),
            &goals,
            &plan,
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
        self.mcp_host = driver::mount_mcp().await;
        self.banner();
        if startup_brief_enabled() {
            if let Ok(lines) = commands::brief_startup(&self.store, &self.cwd) {
                for line in lines {
                    println!("{line}");
                }
            }
        }

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
            if let Some(argument) = input.strip_prefix("/resume ") {
                let Some(index) = parse_resume_index(argument) else {
                    println!("用法：/resume <n>，编号见 /threads");
                    continue;
                };
                let Some(path) = commands::thread_at(&self.store, &thread.path, index)? else {
                    println!("没有第 {index} 条主线；先看 /threads。");
                    continue;
                };
                if path == thread.path {
                    println!("当前就是第 {index} 条。");
                    continue;
                }
                let meta = SessionMeta {
                    cwd: &self.cwd,
                    model: &self.model,
                    provider: &self.provider_id,
                    policy: self.policy,
                    approval: self.approval,
                };
                match driver::open_thread(&path, &meta) {
                    Ok(resumed) => {
                        driver::append_end(&thread.log)?;
                        thread = resumed;
                        history = driver::history(&thread.path)?;
                        println!("resumed {}", thread.path.display());
                    }
                    Err(error) => eprintln!("cannot resume {}: {error:#}", path.display()),
                }
                continue;
            }
            if input == "/new" {
                driver::append_end(&thread.log)?;
                if let Some(archived) =
                    driver::archive_thread(&thread.path, &self.store.threads_dir())?
                {
                    println!("archived {}", archived.display());
                }
                let active_path = self.store.session_path();
                if thread.path != active_path {
                    if let Some(archived) =
                        driver::archive_thread(&active_path, &self.store.threads_dir())?
                    {
                        println!("archived stale active {}", archived.display());
                    }
                }
                thread = {
                    let meta = SessionMeta {
                        cwd: &self.cwd,
                        model: &self.model,
                        provider: &self.provider_id,
                        policy: self.policy,
                        approval: self.approval,
                    };
                    driver::open_thread(&active_path, &meta)?
                };
                history.clear();
                self.rebuild_prompt()?;
                println!("new thread {}", thread.path.display());
                continue;
            }
            if input.starts_with('/') {
                match self.command(input, &thread) {
                    Ok(Flow::Quit) => break,
                    Ok(Flow::Continue) => {}
                    Err(error) => eprintln!("error: {error:#}"),
                }
                continue;
            }

            let mut config = driver::agent_config(
                &self.provider_id,
                &self.model,
                &self.cwd,
                self.prompt.clone(),
                self.policy,
                self.approval,
                self.allow_unsandboxed,
                self.max_steps,
            );
            config.extra_tools = crate::memory_tool::tools(self.store.root().to_path_buf());
            driver::apply_plan_mode(&mut config, self.plan_mode);
            config.mcp_host = self.mcp_host.clone();
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
            let usage = renderer.usage();
            if usage.input + usage.output + usage.cached > 0 {
                self.metric(
                    "tokens",
                    &format!("{},{},{}", usage.input, usage.output, usage.cached),
                );
            }
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
        if let Some(host) = &self.mcp_host {
            let _ = host.shutdown().await;
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
        let open_goals = self
            .store
            .goals()
            .unwrap_or_default()
            .iter()
            .filter(|goal| !goal.done)
            .count();

        println!(
            "niuse · {} · {} / {}{}",
            self.model,
            self.policy.mode.as_str(),
            self.approval.as_str(),
            if self.plan_mode { " · plan" } else { "" }
        );
        println!(
            "{} 条记忆（{} 条待验证）· {} 个进行中的目标 · {} 条未完成承诺 · prompt ~{} tokens",
            memories.len(),
            stale,
            open_goals,
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
                    commands::mem_search(&self.store, query.trim(), false, false, None, None)?;
                } else if let Some(id) = argument.strip_prefix("show ") {
                    commands::mem_show(&self.store, id.trim(), false)?;
                } else {
                    commands::mem_list(&self.store, false, false, None, None)?;
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
            "/threads" => commands::threads(&self.store, &thread.path, false)?,
            "/sessions" => commands::sessions(&self.store, false)?,
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
            "/recall" => {
                if argument.is_empty() {
                    println!("用法：/recall <关键词>");
                } else {
                    print!("{}", commands::recall_text(&self.store, argument, 20)?);
                }
            }
            "/goals" => {
                if let Some(text) = argument.strip_prefix("add ") {
                    commands::goals_add(&self.store, text.trim())?;
                } else if let Some(id) = argument.strip_prefix("done ") {
                    commands::goals_done(&self.store, id.trim())?;
                } else {
                    commands::goals_list(&self.store, false, false)?;
                }
            }
            "/loops" => commands::loops_list(&self.store, false, false, false)?,
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
                    let path = expand_tilde(argument);
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
            "/cost" => {
                let totals = usage_totals(&load(&thread.path)?.events);
                let mut line = format!(
                    "session: {} turn{} · {} in · {} out",
                    totals.turns,
                    if totals.turns == 1 { "" } else { "s" },
                    render::compact_tokens(totals.input),
                    render::compact_tokens(totals.output)
                );
                if totals.cached > 0 {
                    line.push_str(&format!(
                        " · {} cached",
                        render::compact_tokens(totals.cached)
                    ));
                }
                println!("{line}");
            }
            "/plan" => match argument.trim() {
                "" => println!(
                    "plan mode: {}（工具：read/grep/glob）",
                    if self.plan_mode { "on" } else { "off" }
                ),
                "on" => {
                    self.plan_mode = true;
                    println!("plan mode: on（只读；/plan off 退出）");
                }
                "off" | "approve" => {
                    self.plan_mode = false;
                    println!("plan mode: off");
                }
                _ => println!("用法：/plan [on|off]"),
            },
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
            "/brief" => print!("{}", commands::brief_text(&self.store, &self.cwd)?),
            "/evidence" => {
                print!("{}", crate::evidence::from_thread(&thread.path)?.render());
            }
            "/report" => {
                let days = argument.parse().unwrap_or(7);
                let dogfood = self.cwd.join("docs/dogfood.md");
                let dogfood = dogfood.is_file().then_some(dogfood);
                print!("{}", commands::report_text(&self.store, days, dogfood)?);
            }
            "/verify" => {
                if argument.is_empty() {
                    println!("用法：/verify <命令>");
                } else if !commands::verify_shell(&thread.path, argument, false)?.ok {
                    println!("verify failed");
                }
            }
            "/status" => {
                commands::status(&self.store, &thread.path, &self.cwd, false)?;
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
                    commands::user_show(&self.store, false)?;
                }
            }
            "/identity" => {
                if argument == "edit" {
                    commands::identity_edit(&self.store)?;
                } else {
                    commands::identity_show(&self.store, false)?;
                }
            }
            "/prompt" => println!("{}", self.prompt),
            other => println!("未知命令 {other}；/help 查看可用命令。"),
        }
        Ok(Flow::Continue)
    }

    fn handle_blocks(&mut self, blocks: Vec<Block>, source: &str) -> Result<()> {
        if self.plan_mode {
            if !blocks.is_empty() {
                println!("plan mode：忽略本轮写入块。");
            }
            return Ok(());
        }
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

    fn suppress_rejected(&mut self, tags: &[String]) -> bool {
        if suppressed_by_tag(tags, &self.rejected_tags) {
            println!("候选含本 session 已拒绝的 tag，跳过。");
            self.metric("suppress", &tags.join(","));
            return true;
        }
        false
    }

    fn remember_rejected(&mut self, tags: &[String]) {
        for tag in tags {
            self.rejected_tags.insert(tag.to_lowercase());
        }
    }

    fn offer_memory(
        &mut self,
        kind: MemoryKind,
        tags: Vec<String>,
        text: String,
        source: &str,
    ) -> Result<()> {
        if self.suppress_rejected(&tags) {
            return Ok(());
        }
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
                    self.remember_rejected(&tags);
                    self.metric("discard", &tags.join(","));
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
                self.remember_rejected(&tags);
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

pub async fn run_exec(mut app: App, prompt: String) -> Result<()> {
    app.mcp_host = driver::mount_mcp().await;
    let log = driver::new_session(&app.store.root().join("sessions"))?;
    let meta = SessionMeta {
        cwd: &app.cwd,
        model: &app.model,
        provider: &app.provider_id,
        policy: app.policy,
        approval: app.approval,
    };
    driver::append_start(&log, &meta)?;

    let mut config = driver::agent_config(
        &app.provider_id,
        &app.model,
        &app.cwd,
        app.prompt.clone(),
        app.policy,
        app.approval,
        app.allow_unsandboxed,
        app.max_steps,
    );
    config.extra_tools = crate::memory_tool::tools(app.store.root().to_path_buf());
    driver::apply_plan_mode(&mut config, app.plan_mode);
    config.mcp_host = app.mcp_host.clone();
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
    let usage = renderer.usage();
    if usage.input + usage.output + usage.cached > 0 {
        app.metric(
            "tokens",
            &format!("{},{},{}", usage.input, usage.output, usage.cached),
        );
    }

    if app.plan_mode {
        if !parsed.blocks.is_empty() {
            eprintln!(
                "plan mode: ignored {} protocol block(s)",
                parsed.blocks.len()
            );
        }
    } else {
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
    }

    let status = match &result {
        Ok(Some(_)) => "ok",
        Ok(None) => "cancelled",
        Err(_) => "error",
    };
    app.metric("turn", status);

    driver::append_end(&log)?;
    if let Some(host) = &app.mcp_host {
        let _ = host.shutdown().await;
    }
    println!("\nsession: {}", log.path().display());
    result.map(|_| ())
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct UsageTotals {
    pub(crate) turns: usize,
    pub(crate) input: u64,
    pub(crate) output: u64,
    pub(crate) cached: u64,
}

pub(crate) fn usage_totals(events: &[Event]) -> UsageTotals {
    let mut totals = UsageTotals::default();
    for event in events {
        match event {
            Event::UserMessage { .. } => totals.turns += 1,
            Event::TurnEnd { usage, .. } => {
                totals.input += usage.input_tokens;
                totals.output += usage.output_tokens;
                totals.cached += usage.cached_tokens;
            }
            _ => {}
        }
    }
    totals
}

fn expand_tilde(input: &str) -> PathBuf {
    if input == "~" {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("~"));
    }
    if let Some(rest) = input.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(input)
}

fn parse_resume_index(argument: &str) -> Option<usize> {
    match argument.trim().parse::<usize>() {
        Ok(index) if index > 0 => Some(index),
        _ => None,
    }
}

fn suppressed_by_tag(tags: &[String], rejected: &HashSet<String>) -> bool {
    tags.iter()
        .any(|tag| rejected.contains(&tag.to_lowercase()))
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
        let path = expand_tilde(argument);
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

const HELP: &str = r#"对话
  直接输入一条消息；空行会被忽略。终端里行尾加 \ 可续行。

记忆
  /mem                    列出现行记忆
  /mem recall <query>     检索记忆，包含已过期条目
  /mem show <id>          查看单条记忆的全部字段
  /mem review             复核过期条目（y 保留 / e 编辑 / d 删除 / s 跳过）
  /recall <query>         跨记忆、日记、承诺检索（含已过期）
  /remember <一句话>      直接记住一条偏好
  /forget <id>            删除一条记忆

承诺与日记
  /goals                  列出进行中的目标
  /goals add <text>       添加一个目标（由你维护，最多 7 个）
  /goals done <g-id>      完成一个目标
  /loops                  列出承诺
  /loop <text>            记下一条承诺
  /done <l-id>            关闭一条承诺
  /journal                最近 7 天
  /journal search <query> 检索日记条目
  /journal add <text>     追加一条今天的日记

会话
  /brief                  今天需要处理什么（逾期/审查/过期）
  /evidence               本 thread 的工具调用、文件、检查与失败
  /report [days]          周报：turn、成本、store、dogfood
  /verify <command>       运行检查并把结果写入 thread
  /plan [on|off]          只读计划模式（工具限制为 read/grep/glob）
  /export [path]          导出当前主线为 Markdown
  /new                    归档当前主线并开始一条新的
  /cd <path>              切换工作目录（下一个 turn 生效）
  /status                 当前配置与存储状态

模型与成本
  /model [name]           查看或切换模型（default 回退）
  /cost                   本次会话累计 token
  /stats                  最近 7 天的候选接受率与活动计数

身份与设置
  /identity               查看 identity.md（/identity edit 编辑）
  /user                   查看 user.md（/user edit 编辑）
  /doctor                 检查本地设置，不调用模型

调试
  /prompt                 打印完整 system prompt
  /help                   本帮助
  /quit                   退出
"#;

fn print_help() {
    print!("{HELP}");
}

fn startup_brief_enabled() -> bool {
    brief_enabled(std::env::var("NIUSE_BRIEF").ok().as_deref())
}

fn brief_enabled(value: Option<&str>) -> bool {
    value
        .map(|value| value.trim().to_lowercase() != "off")
        .unwrap_or(true)
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
    fn brief_can_be_disabled_at_startup() {
        assert!(brief_enabled(None));
        assert!(brief_enabled(Some("on")));
        assert!(!brief_enabled(Some("off")));
        assert!(!brief_enabled(Some(" OFF ")));
    }

    #[test]
    fn usage_totals_counts_turns() {
        let events: Vec<Event> = vec![
            serde_json::from_str(r#"{"type":"user_message","text":"first","ts":0}"#).unwrap(),
            serde_json::from_str(
                r#"{"type":"turn_end","usage":{"input_tokens":100,"output_tokens":10,"cached_tokens":40},"ts":1}"#,
            )
            .unwrap(),
            serde_json::from_str(
                r#"{"type":"turn_end","usage":{"input_tokens":50,"output_tokens":5,"cached_tokens":0},"ts":2}"#,
            )
            .unwrap(),
            serde_json::from_str(r#"{"type":"user_message","text":"second","ts":3}"#).unwrap(),
        ];
        let totals = usage_totals(&events);
        assert_eq!(totals.turns, 2);
        assert_eq!(totals.input, 150);
        assert_eq!(totals.output, 15);
        assert_eq!(totals.cached, 40);
    }

    #[test]
    fn tilde_expands_against_home() {
        if let Some(home) = std::env::var_os("HOME") {
            assert_eq!(
                expand_tilde("~/out.md"),
                PathBuf::from(home.clone()).join("out.md")
            );
            assert_eq!(expand_tilde("~"), PathBuf::from(home));
        }
        assert_eq!(expand_tilde("plain.md"), PathBuf::from("plain.md"));
    }

    #[test]
    fn resume_index_is_one_based_and_strict() {
        assert_eq!(parse_resume_index("1"), Some(1));
        assert_eq!(parse_resume_index(" 2 "), Some(2));
        assert_eq!(parse_resume_index("0"), None);
        assert_eq!(parse_resume_index("abc"), None);
        assert_eq!(parse_resume_index(""), None);
        assert_eq!(parse_resume_index("-1"), None);
    }

    #[test]
    fn rejected_tags_hide_later_candidates() {
        let mut rejected = HashSet::new();
        rejected.insert("rust".to_string());
        assert!(suppressed_by_tag(&["RUST".to_string()], &rejected));
        assert!(!suppressed_by_tag(&["style".to_string()], &rejected));
        assert!(!suppressed_by_tag(&[], &rejected));
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

        let home = path_candidates("~", directory.path(), PathScope::Any);
        assert_eq!(home.len(), 1);
        assert_eq!(home[0].replacement, "~/");
        let up = path_candidates("..", directory.path(), PathScope::Any);
        assert_eq!(up.len(), 1);
        assert_eq!(up[0].replacement, "../");
        assert_eq!(
            path_candidates(".", directory.path(), PathScope::Any)[0].replacement,
            "./"
        );

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

        let prefixed = id_candidates(&store, IdScope::Memory, &format!("m-{}", first.id));
        assert_eq!(prefixed.len(), 1);
        assert_eq!(prefixed[0].replacement, format!("m-{}", first.id));

        let loop_item = store.add_loop(LoopItem::new("open loop", None)).unwrap();
        let loops = id_candidates(&store, IdScope::Loop, "l-");
        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].replacement, format!("l-{}", loop_item.id));
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
