//! Interactive REPL and one-shot execution.

use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use niucode_agent::ApprovalMode;
use niucode_core::load;
use niucode_provider::Provider;
use niucode_tools::SandboxPolicy;
use niuse_core::policy::{conflicts, secret_reason};
use niuse_core::prompt::compose;
use niuse_core::{Block, Date, LoopItem, MemoryEntry, MemoryKind, Store};

use crate::commands;
use crate::config::{build_provider, resolve_cwd, resolve_policy};
use crate::driver::{self, SessionMeta, Thread};
use crate::render::{self, Renderer};
use crate::Cli;

pub struct App {
    pub store: Store,
    pub provider: Arc<dyn Provider>,
    pub provider_id: String,
    pub model: String,
    pub policy: SandboxPolicy,
    pub approval: ApprovalMode,
    pub allow_unsandboxed: bool,
    pub cwd: PathBuf,
    pub max_steps: usize,
    pub prompt: String,
    pub prompt_tokens: usize,
    memory_enabled: bool,
    suppress_candidates: bool,
}

impl App {
    pub fn new(cli: &Cli, store: Store) -> Result<Self> {
        let bundle = build_provider(&cli.preset, cli.model.as_deref(), cli.no_thinking)?;
        let (policy, approval, allow_unsandboxed) =
            resolve_policy(&cli.sandbox, &cli.approval, &cli.network)?;
        let cwd = resolve_cwd(cli.cwd.clone())?;
        let memory_enabled = std::env::var("NIUSE_MEMORY")
            .map(|value| value.trim().to_lowercase() != "off")
            .unwrap_or(true);

        let mut app = Self {
            store,
            provider: bundle.provider,
            provider_id: bundle.provider_id,
            model: bundle.model,
            policy,
            approval,
            allow_unsandboxed,
            cwd,
            max_steps: cli.max_steps,
            prompt: String::new(),
            prompt_tokens: 0,
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
        let path = self.store.session_path();
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

        loop {
            print!("\n{}", self.prompt_label());
            let _ = io::stdout().flush();
            let Some(line) = render::read_line() else {
                println!();
                break;
            };
            let input = line.trim();
            if input.is_empty() {
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
                    commands::mem_search(&self.store, query.trim(), false)?;
                } else if let Some(id) = argument.strip_prefix("show ") {
                    commands::mem_show(&self.store, id.trim())?;
                } else {
                    commands::mem_list(&self.store, false)?;
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
            "/journal" => commands::journal_show(&self.store, 7)?,
            "/stats" => commands::stats(&self.store, 7)?,
            "/doctor" => {
                if let Err(error) = commands::doctor(&self.store, &self.provider_id) {
                    println!("{error}");
                }
            }
            "/loops" => commands::loops_list(&self.store, false)?,
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
                    println!("cwd: {}", self.cwd.display());
                }
            }
            "/status" => {
                commands::status(&self.store)?;
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
            "/identity" => commands::identity_show(&self.store)?,
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

    driver::append_end(&log)?;
    println!("\nsession: {}", log.path().display());
    result.map(|_| ())
}

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
         /stats          最近 7 天的候选接受率与活动计数\n\
         /doctor         检查本地设置，不调用模型\n\
         /identity       查看 identity.md\n\
         /cd <path>      切换工作目录（下一个 turn 生效）\n\
         /status         当前配置与存储状态\n\
         /prompt         打印完整 system prompt（调试用）\n\
         /help           本帮助\n\
         /quit           退出"
    );
}
