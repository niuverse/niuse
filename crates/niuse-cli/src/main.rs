#![forbid(unsafe_code)]

mod bootstrap;
mod commands;
mod config;
mod digest;
mod driver;
mod evidence;
mod export;
mod memory_tool;
mod projects;
mod render;
mod repl;
mod settings;
#[cfg(feature = "tui")]
mod tui_hook;

use std::path::PathBuf;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use niuse_core::Store;
#[cfg(feature = "web")]
use serde_json::json;

#[derive(Parser, Debug)]
#[command(
    name = "niuse",
    version,
    about = "A personal engineering companion built on niucode"
)]
struct Cli {
    /// Provider preset (deepseek or openai); defaults from config
    #[arg(long, global = true)]
    preset: Option<String>,
    /// Override the preset model
    #[arg(long, global = true)]
    model: Option<String>,
    /// Disable DeepSeek thinking mode
    #[arg(long, global = true)]
    no_thinking: bool,
    /// Working directory (defaults to cwd); `/cd` changes it at runtime
    #[arg(long, global = true)]
    cwd: Option<PathBuf>,
    /// Sandbox: read-only, workspace-write, or full-access; defaults from config
    #[arg(long, global = true)]
    sandbox: Option<String>,
    /// Approval: ask, on-failure, or never; defaults from config
    #[arg(long, global = true)]
    approval: Option<String>,
    /// Network: on or off; defaults from config
    #[arg(long, global = true)]
    network: Option<String>,
    /// Override the niuse home (defaults to $NIUSE_HOME or ~/.niuse)
    #[arg(long, global = true)]
    home: Option<PathBuf>,
    /// Thread JSONL to resume (defaults to ~/.niuse/thread.jsonl)
    #[arg(long, global = true)]
    thread: Option<PathBuf>,
    /// Maximum model turns per user request (0 = no hard cap); defaults from config
    #[arg(long, global = true)]
    max_steps: Option<usize>,
    /// Start in read-only plan mode (read/grep/glob)
    #[arg(long, global = true)]
    plan: bool,
    /// Native desktop policy: off, observe, control, or autonomous
    #[arg(long, global = true, default_value = "off", value_parser = ["off", "observe", "control", "autonomous"])]
    computer: String,
    /// Mount MCP servers whose manifest has read_only = false and write_tools
    #[arg(long, global = true)]
    allow_mcp_write: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone)]
enum McpCommand {
    /// Write the bundled read-only packs and a trust manifest
    Init {
        /// Notes directory for the notes pack
        #[arg(long)]
        notes: Option<PathBuf>,
        /// iCalendar file for the calendar pack
        #[arg(long)]
        calendar: Option<PathBuf>,
        /// Manifest path (defaults to ~/.niucode/mcp.toml)
        #[arg(long)]
        config: Option<PathBuf>,
        /// Where to write the bundled server scripts (defaults to ~/.niuse/packs)
        #[arg(long)]
        packs: Option<PathBuf>,
        /// Replace an existing manifest (no merge)
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum PlanCommand {
    /// List open steps (or all with --all)
    List {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        json: bool,
    },
    /// Add one step
    Add { text: String },
    /// Close a step; requires a passing verification for p-N
    Done {
        id: u32,
        /// Thread containing the verification (defaults to the main thread)
        #[arg(long)]
        thread: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Run one prompt through the agent, then exit
    Exec {
        prompt: String,
        /// Text file to attach to this turn (repeatable)
        #[arg(long, value_name = "PATH")]
        attach: Vec<PathBuf>,
        /// Image file to attach to this turn (repeatable)
        #[arg(long, value_name = "PATH")]
        image: Vec<PathBuf>,
    },
    /// Inspect and edit durable memory
    Mem {
        #[command(subcommand)]
        command: MemCommand,
    },
    /// Read-only weekly review across metrics, stores, and dogfood
    Report {
        /// Window in days
        #[arg(long, default_value_t = 7)]
        days: u64,
        /// Dogfood table (defaults to docs/dogfood.md when present)
        #[arg(long)]
        dogfood: Option<PathBuf>,
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },
    /// Record today's dogfood row from local metrics
    Dogfood {
        /// Markdown file with the dogfood table (defaults to ~/.niuse/dogfood.md)
        #[arg(long)]
        file: Option<PathBuf>,
        /// One concrete note for the day
        #[arg(long)]
        note: Option<String>,
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },
    /// Inspect and close open loops
    Loops {
        #[command(subcommand)]
        command: LoopsCommand,
    },
    /// Manage human-owned goals
    Goals {
        #[command(subcommand)]
        command: GoalsCommand,
    },
    /// Export a session log as Markdown
    Export {
        /// Write to a file instead of stdout
        #[arg(long)]
        out: Option<PathBuf>,
        /// Session JSONL to export (defaults to the main thread)
        #[arg(long)]
        session: Option<PathBuf>,
    },
    /// List the active thread and archived chapters
    Threads {
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },
    /// Serve the local web companion (identity/memory prompt, browser chat)
    #[cfg(feature = "web")]
    Web {
        /// Loopback port (0 chooses an ephemeral port)
        #[arg(long, default_value_t = 8788)]
        port: u16,
    },
    /// Experimental: run niucode's TUI with niuse hooks
    #[cfg(feature = "tui")]
    Tui {
        /// Replay normalized provider turns from a fixture JSON (testing)
        #[arg(long)]
        replay: Option<PathBuf>,
    },
    /// Generate shell completions
    Completions {
        /// Target shell
        shell: clap_complete::Shell,
    },
    /// List one-shot `exec` session logs
    Sessions {
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },
    /// Show or search the journal
    Journal {
        #[arg(long, default_value_t = 7)]
        days: usize,
        /// Search entries instead of listing recent days
        #[arg(long)]
        search: Option<String>,
        /// Append an entry to today instead of listing
        #[arg(long)]
        add: Option<String>,
        /// Emit JSON for show or search
        #[arg(long)]
        json: bool,
    },
    /// Create, show, or edit identity.md
    Identity {
        #[command(subcommand)]
        command: IdentityCommand,
    },
    /// Mine sessions for memory candidates or a user profile
    Bootstrap {
        /// Maximum number of sessions to scan
        #[arg(long, default_value_t = 40)]
        limit: usize,
        /// Only sessions modified within this many days
        #[arg(long, default_value_t = 90)]
        since_days: i64,
        /// List the sessions that would be scanned, without calling a model
        #[arg(long)]
        dry_run: bool,
        /// Also draft user.md from the selected sessions
        #[arg(long)]
        profile: bool,
        /// Overwrite an existing user.md when drafting
        #[arg(long)]
        force_profile: bool,
        /// Draft user.md only; skip memory candidates
        #[arg(long)]
        profile_only: bool,
    },
    /// Review bootstrapped candidates
    Review,
    /// Summarize the current thread into the journal
    Digest {
        /// Session JSONL to digest (defaults to the main thread)
        #[arg(long)]
        thread: Option<PathBuf>,
        /// Print the bullets without writing
        #[arg(long)]
        dry_run: bool,
        /// Maximum number of recent user/assistant messages to read
        #[arg(long, default_value_t = 40)]
        limit: usize,
    },
    /// Check local setup without calling a model
    Doctor {
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },
    /// Inspect or edit user.md
    User {
        #[command(subcommand)]
        command: UserCommand,
    },
    /// Inspect or create ~/.niuse/config.toml
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Show local acceptance counters
    Stats {
        /// Window in local calendar days; 0 means all time
        #[arg(long, default_value_t = 7)]
        days: u64,
        /// Add a per-day breakdown
        #[arg(long)]
        trend: bool,
        /// Emit JSON (summary plus per-day rows)
        #[arg(long)]
        json: bool,
    },
    /// Install read-only MCP packs and write a trust manifest
    Mcp {
        #[command(subcommand)]
        command: McpCommand,
    },
    /// Human-owned plan steps, with a verification gate for closing
    Plan {
        #[command(subcommand)]
        command: PlanCommand,
    },
    /// Run a verification command and append its result to the thread
    Verify {
        /// Thread JSONL to append to (defaults to the main thread)
        #[arg(long)]
        thread: Option<PathBuf>,
        /// Emit JSON
        #[arg(long)]
        json: bool,
        /// Plan step this check verifies, for example p-2
        #[arg(long)]
        step: Option<String>,
        /// Command and arguments, after `--`
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },
    /// Show tool calls, files, checks, and failures from a thread
    Evidence {
        /// Thread JSONL to inspect (defaults to the main thread)
        #[arg(long)]
        thread: Option<PathBuf>,
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },
    /// Show what needs attention today
    Brief {
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },
    /// Search memory, journal, and loops
    Recall {
        /// Search terms
        query: String,
        /// Maximum number of hits
        #[arg(long, default_value_t = 20)]
        limit: usize,
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },
    /// List recent niucode projects
    Projects {
        /// Maximum number to show
        #[arg(long, default_value_t = 10)]
        limit: usize,
        /// Include missing and temporary projects
        #[arg(long)]
        all: bool,
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },
    /// Show store and prompt status
    Status {
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum MemCommand {
    /// List memories
    List {
        /// Include expired entries
        #[arg(long)]
        all: bool,
        /// Emit JSON
        #[arg(long)]
        json: bool,
        /// Only preference or lesson
        #[arg(long)]
        kind: Option<String>,
        /// Only entries carrying this tag
        #[arg(long)]
        tag: Option<String>,
    },
    /// Add a memory directly
    Add {
        text: String,
        /// preference or lesson
        #[arg(long, default_value = "preference")]
        kind: String,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        /// Store even if the secret scan or duplicate check matches
        #[arg(long)]
        force: bool,
    },
    /// Search memories, including expired ones
    Search {
        query: String,
        /// Exclude expired entries
        #[arg(long)]
        fresh: bool,
        /// Emit JSON
        #[arg(long)]
        json: bool,
        /// Only preference or lesson
        #[arg(long)]
        kind: Option<String>,
        /// Only entries carrying this tag
        #[arg(long)]
        tag: Option<String>,
    },
    /// Show one memory in full
    Show {
        id: String,
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },
    /// Edit a memory in $EDITOR
    Edit { id: String },
    /// Delete a memory
    Forget { id: String },
    /// Refresh the confirmation date
    Confirm { id: String },
    /// Review stale memories interactively
    Review,
    /// Remove memories older than 90 days (dry run by default)
    Prune {
        /// Actually delete the expired entries
        #[arg(long)]
        apply: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum GoalsCommand {
    /// List goals
    List {
        /// Include completed goals
        #[arg(long)]
        all: bool,
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },
    /// Add a goal manually
    Add { text: String },
    /// Complete a goal
    Done { id: String },
}

#[derive(Subcommand, Debug, Clone)]
enum LoopsCommand {
    /// List loops
    List {
        /// Include closed loops
        #[arg(long)]
        all: bool,
        /// Emit JSON
        #[arg(long)]
        json: bool,
        /// Only overdue loops
        #[arg(long)]
        overdue: bool,
    },
    /// Add a loop manually
    Add {
        text: String,
        /// Optional due date (YYYY-MM-DD)
        #[arg(long)]
        due: Option<String>,
    },
    /// Close a loop
    Done { id: String },
}

#[derive(Subcommand, Debug, Clone)]
enum UserCommand {
    /// Print user.md
    Show {
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },
    /// Edit user.md in $EDITOR
    Edit,
}

#[derive(Subcommand, Debug, Clone)]
enum ConfigCommand {
    /// Write a commented template
    Init {
        /// Overwrite an existing config
        #[arg(long)]
        force: bool,
    },
    /// Print the config file
    Show,
    /// Print the merged effective values
    Effective {
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum IdentityCommand {
    /// Answer eight questions and write identity.md
    Init {
        /// Overwrite an existing identity.md
        #[arg(long)]
        force: bool,
    },
    /// Print identity.md
    Show {
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },
    /// Edit identity.md in $EDITOR
    Edit,
}

/// Turn a closed stdout into a clean exit.
///
/// Rust ignores SIGPIPE, so `niuse ... | head` panics when the reader closes
/// early. Restoring SIGPIPE globally would also affect network writes; a panic
/// hook scoped to this one message is safer.
fn install_pipe_guard() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let broken = info
            .payload()
            .downcast_ref::<&str>()
            .is_some_and(|message| message.contains("Broken pipe"))
            || info
                .payload()
                .downcast_ref::<String>()
                .is_some_and(|message| message.contains("Broken pipe"));
        if broken {
            std::process::exit(0);
        }
        default(info);
    }));
}

#[tokio::main]
async fn main() -> Result<()> {
    install_pipe_guard();
    let cli = Cli::parse();
    let command = cli.command.clone();
    if let Some(Command::Completions { shell }) = &command {
        let mut command = Cli::command();
        clap_complete::generate(*shell, &mut command, "niuse", &mut std::io::stdout());
        return Ok(());
    }

    let store = match &cli.home {
        Some(path) => Store::at(path.clone()),
        None => Store::discover()?,
    };
    store.init()?;

    match command {
        #[cfg(feature = "web")]
        Some(Command::Web { port }) => run_web(&cli, &store, port).await,
        #[cfg(feature = "tui")]
        Some(Command::Tui { replay }) => run_tui(&cli, &store, replay).await,
        Some(Command::Completions { .. }) => Ok(()),
        None => {
            let settings = settings::resolve(&cli, &store)?;
            let app = repl::App::new(&cli, &settings, store)?;
            app.run().await
        }
        Some(Command::Exec {
            prompt,
            attach,
            image,
        }) => {
            let settings = settings::resolve(&cli, &store)?;
            let app = repl::App::new(&cli, &settings, store)?;
            repl::run_exec(app, prompt, attach, image).await
        }
        Some(Command::Mem { command }) => match command {
            MemCommand::List {
                all,
                json,
                kind,
                tag,
            } => commands::mem_list(&store, all, json, kind.as_deref(), tag.as_deref()),
            MemCommand::Add {
                text,
                kind,
                tags,
                force,
            } => commands::mem_add(&store, &text, &kind, tags, force),
            MemCommand::Search {
                query,
                fresh,
                json,
                kind,
                tag,
            } => commands::mem_search(&store, &query, fresh, json, kind.as_deref(), tag.as_deref()),
            MemCommand::Show { id, json } => commands::mem_show(&store, &id, json),
            MemCommand::Edit { id } => commands::mem_edit(&store, &id),
            MemCommand::Forget { id } => commands::mem_forget(&store, &id),
            MemCommand::Confirm { id } => commands::mem_confirm(&store, &id),
            MemCommand::Review => commands::mem_review(&store),
            MemCommand::Prune { apply } => commands::mem_prune(&store, apply),
        },
        Some(Command::Report {
            days,
            dogfood,
            json,
        }) => {
            let dogfood = dogfood.or_else(|| {
                let path = PathBuf::from("docs/dogfood.md");
                path.is_file().then_some(path)
            });
            commands::report(&store, days, dogfood, json)
        }
        Some(Command::Dogfood { file, note, json }) => commands::dogfood(&store, file, note, json),
        Some(Command::Loops { command }) => match command {
            LoopsCommand::List { all, json, overdue } => {
                commands::loops_list(&store, all, json, overdue)
            }
            LoopsCommand::Add { text, due } => commands::loops_add(&store, &text, due.as_deref()),
            LoopsCommand::Done { id } => commands::loops_done(&store, &id),
        },
        Some(Command::Goals { command }) => match command {
            GoalsCommand::List { all, json } => commands::goals_list(&store, all, json),
            GoalsCommand::Add { text } => commands::goals_add(&store, &text),
            GoalsCommand::Done { id } => commands::goals_done(&store, &id),
        },
        Some(Command::Sessions { json }) => commands::sessions(&store, json),
        Some(Command::Threads { json }) => {
            let active = config::resolve_thread(cli.thread.clone(), &store)?;
            commands::threads(&store, &active, json)
        }
        Some(Command::Export { out, session }) => {
            commands::export(&store, out.as_deref(), session.as_deref())
        }
        Some(Command::Journal {
            days,
            search,
            add,
            json,
        }) => {
            if let Some(text) = add {
                commands::journal_add(&store, &text)
            } else if let Some(query) = search {
                commands::journal_search(&store, &query, json)
            } else {
                commands::journal_show(&store, days, json)
            }
        }
        Some(Command::Identity { command }) => match command {
            IdentityCommand::Init { force } => commands::identity_init(&store, force),
            IdentityCommand::Show { json } => commands::identity_show(&store, json),
            IdentityCommand::Edit => commands::identity_edit(&store),
        },
        Some(Command::Bootstrap {
            limit,
            since_days,
            dry_run,
            profile,
            force_profile,
            profile_only,
        }) => {
            let settings = settings::resolve(&cli, &store)?;
            let app = repl::App::new(&cli, &settings, store)?;
            bootstrap::run(
                &app.store,
                app.provider.as_ref(),
                &app.model,
                bootstrap::BootstrapOptions {
                    limit,
                    since_days,
                    dry_run,
                    profile,
                    force_profile,
                    profile_only,
                },
            )
            .await
        }
        Some(Command::Review) => bootstrap::review(&store),
        Some(Command::Digest {
            thread,
            dry_run,
            limit,
        }) => {
            let settings = settings::resolve(&cli, &store)?;
            let thread = config::resolve_thread(thread.or(cli.thread.clone()), &store)?;
            let cwd = config::resolve_cwd(cli.cwd.clone())?;
            let project = cwd
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "unknown".to_string());
            let app = repl::App::new(&cli, &settings, store)?;
            digest::run(
                &app.store,
                app.provider.as_ref(),
                &app.model,
                &thread,
                &project,
                digest::DigestOptions { dry_run, limit },
            )
            .await
        }
        Some(Command::Doctor { json }) => {
            let settings = settings::resolve(&cli, &store)?;
            commands::doctor(&store, &settings.provider, json)
        }
        Some(Command::User { command }) => match command {
            UserCommand::Show { json } => commands::user_show(&store, json),
            UserCommand::Edit => commands::user_edit(&store),
        },
        Some(Command::Config { command }) => match command {
            ConfigCommand::Init { force } => {
                if settings::init(&store, force)? {
                    println!("wrote {}", store.config_path().display());
                } else {
                    println!(
                        "{} already exists; pass --force to overwrite",
                        store.config_path().display()
                    );
                }
                Ok(())
            }
            ConfigCommand::Show => settings::show(&store),
            ConfigCommand::Effective { json } => {
                let settings = settings::resolve(&cli, &store)?;
                if json {
                    let payload = serde_json::json!({
                        "provider": settings.provider,
                        "model": settings.model,
                        "sandbox": settings.sandbox,
                        "approval": settings.approval,
                        "network": settings.network,
                        "max_steps": settings.max_steps,
                    });
                    println!("{}", serde_json::to_string_pretty(&payload)?);
                    return Ok(());
                }
                println!("provider    {}", settings.provider);
                println!(
                    "model       {}",
                    settings.model.as_deref().unwrap_or("(provider default)")
                );
                println!("sandbox     {}", settings.sandbox);
                println!("approval    {}", settings.approval);
                println!("network     {}", settings.network);
                println!(
                    "max_steps   {}",
                    if settings.max_steps == 0 {
                        "unlimited".to_string()
                    } else {
                        settings.max_steps.to_string()
                    }
                );
                Ok(())
            }
        },
        Some(Command::Stats { days, trend, json }) => commands::stats(&store, days, trend, json),
        Some(Command::Mcp { command }) => match command {
            McpCommand::Init {
                notes,
                calendar,
                config,
                packs,
                force,
            } => commands::mcp_init(
                &store,
                commands::McpInitOptions {
                    notes,
                    calendar,
                    config,
                    packs,
                    force,
                },
            ),
        },
        Some(Command::Plan { command }) => match command {
            PlanCommand::List { all, json } => commands::plan_list(&store, all, json),
            PlanCommand::Add { text } => {
                let step = commands::plan_add(&store, &text)?;
                println!("added {}", step.label());
                Ok(())
            }
            PlanCommand::Done { id, thread } => {
                let thread = config::resolve_thread(thread.or(cli.thread.clone()), &store)?;
                let step = commands::plan_done(&store, &thread, id)?;
                println!("done {}", step.label());
                Ok(())
            }
        },
        Some(Command::Verify {
            thread,
            json,
            step,
            command,
        }) => {
            let thread = config::resolve_thread(thread.or(cli.thread.clone()), &store)?;
            let outcome = commands::verify_with_step(&thread, &command, step.as_deref(), json)?;
            std::process::exit(outcome.code);
        }
        Some(Command::Evidence { thread, json }) => {
            let thread = config::resolve_thread(thread.or(cli.thread.clone()), &store)?;
            commands::evidence(&thread, &store, json)
        }
        Some(Command::Brief { json }) => {
            let cwd = config::resolve_cwd(cli.cwd.clone())?;
            commands::brief(&store, &cwd, json)
        }
        Some(Command::Recall { query, limit, json }) => {
            commands::recall(&store, &query, limit, json)
        }
        Some(Command::Projects { limit, all, json }) => commands::projects(limit, all, json),
        Some(Command::Status { json }) => {
            let thread = config::resolve_thread(cli.thread.clone(), &store)?;
            let cwd = config::resolve_cwd(cli.cwd.clone())?;
            commands::status(&store, &thread, &cwd, json)
        }
    }
}

#[cfg(feature = "web")]
async fn run_web(cli: &Cli, store: &Store, port: u16) -> Result<()> {
    let settings = settings::resolve(cli, store)?;
    let cwd = config::resolve_cwd(cli.cwd.clone())?;
    let model = settings
        .model
        .clone()
        .unwrap_or_else(|| "deepseek-flash".to_string());
    let identity = store.identity()?;
    let user = store.user()?;
    let memories = store.memories()?;
    let goals = store.goals()?;
    let plan = store.plan()?;
    let loops = store.loops()?;
    let journal = store.recent_journal(7)?;
    let composition = niuse_core::prompt::compose_with_plan(
        identity.as_deref(),
        user.as_deref(),
        &goals,
        &plan,
        &memories,
        &loops,
        &journal,
        niuse_core::date::Date::today(),
    );
    let context = web_context(store)?;
    niucode::web::serve(niucode::web::Options {
        session: store.session_path(),
        port,
        preset: settings.provider.clone(),
        model: Some(model.clone()),
        cwd: cwd.clone(),
        max_steps: settings.max_steps,
        system_prompt: Some(composition.system),
        brand: Some("niuse".into()),
        context: Some(context),
        context_provider: Some(std::sync::Arc::new({
            let context_store = store.clone();
            move || web_context(&context_store).unwrap_or_else(|_| json!({"brand": "niuse"}))
        })),
        allow_new: true,
        switch_session_provider: Some(std::sync::Arc::new({
            let store = store.clone();
            let cwd = cwd.clone();
            let model = model.clone();
            let provider = settings.provider.clone();
            move |id| web_switch_thread(&store, id, &cwd, &model, &provider)
        })),
        new_session_provider: Some(std::sync::Arc::new({
            let store = store.clone();
            let provider = settings.provider.clone();
            let model = model.clone();
            let cwd = cwd.clone();
            move || web_new_thread(&store, &cwd, &model, &provider)
        })),
    })
    .await
}

#[cfg(feature = "web")]
fn web_new_thread(
    store: &Store,
    cwd: &std::path::Path,
    model: &str,
    provider: &str,
) -> Result<PathBuf> {
    let active = store.session_path();
    driver::archive_thread(&active, &store.threads_dir())?;
    let thread = driver::open_thread(
        &active,
        &driver::SessionMeta {
            cwd,
            model,
            provider,
            policy: niucode_tools::SandboxPolicy::new(
                niucode_tools::SandboxMode::WorkspaceWrite,
                niucode_tools::NetworkMode::Off,
            ),
            approval: niucode_agent::ApprovalMode::Ask,
        },
    )?;
    Ok(thread.path.clone())
}

#[cfg(feature = "web")]
fn web_switch_thread(
    store: &Store,
    id: &str,
    cwd: &std::path::Path,
    model: &str,
    provider: &str,
) -> Result<PathBuf> {
    if id.is_empty()
        || !id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        anyhow::bail!("invalid thread id");
    }
    let selected = store.threads_dir().join(format!("{id}.jsonl"));
    if !selected.exists() {
        anyhow::bail!("thread not found");
    }
    let active = store.session_path();
    driver::archive_thread(&active, &store.threads_dir())?;
    std::fs::rename(&selected, &active)
        .map_err(|error| anyhow::anyhow!("switch {}: {error}", selected.display()))?;
    let thread = driver::open_thread(
        &active,
        &driver::SessionMeta {
            cwd,
            model,
            provider,
            policy: niucode_tools::SandboxPolicy::new(
                niucode_tools::SandboxMode::WorkspaceWrite,
                niucode_tools::NetworkMode::Off,
            ),
            approval: niucode_agent::ApprovalMode::Ask,
        },
    )?;
    Ok(thread.path.clone())
}

#[cfg(feature = "web")]
fn web_context(store: &Store) -> Result<serde_json::Value> {
    let identity = store.identity()?;
    let user = store.user()?;
    let memories = store.memories()?;
    let goals = store.goals()?;
    let plan = store.plan()?;
    let loops = store.loops()?;
    let mut sections = Vec::new();
    sections.push(json!({
        "title": "identity",
        "lines": identity
            .as_deref()
            .map(|text| text.lines().take(10).collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["not set"]),
    }));
    sections.push(json!({
        "title": "user",
        "lines": user
            .as_deref()
            .map(|text| text.lines().take(10).collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["not set"]),
    }));
    sections.push(json!({
        "title": "goals",
        "lines": goals
            .iter()
            .map(|goal| format!("{} · {}", goal.id, goal.text))
            .collect::<Vec<_>>(),
    }));
    sections.push(json!({
        "title": "plan",
        "lines": plan
            .iter()
            .map(|step| format!("{} {} · {}", if step.done { "x" } else { " " }, step.id, step.text))
            .collect::<Vec<_>>(),
    }));
    sections.push(json!({
        "title": "memory",
        "lines": memories
            .iter()
            .take(8)
            .map(|memory| format!("{} · {}", memory.id, memory.text))
            .collect::<Vec<_>>(),
    }));
    sections.push(json!({
        "title": "loops",
        "lines": loops
            .iter()
            .filter(|item| !item.done)
            .take(8)
            .map(|item| format!("{} · {}", item.id, item.text))
            .collect::<Vec<_>>(),
    }));
    let active = store.session_path();
    sections.push(json!({
        "title": "thread",
        "lines": [format!(
            "active · {}",
            active
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("thread.jsonl")
        )],
    }));
    let mut threads: Vec<(std::time::SystemTime, String)> = std::fs::read_dir(store.threads_dir())
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "jsonl"))
                .filter_map(|entry| {
                    let modified = entry.metadata().ok()?.modified().ok()?;
                    let name = entry.path().file_stem()?.to_string_lossy().to_string();
                    Some((modified, name))
                })
                .collect()
        })
        .unwrap_or_default();
    threads.sort_by_key(|right| std::cmp::Reverse(right.0));
    sections.push(json!({
        "title": "threads",
        "kind": "threads",
        "lines": threads
            .into_iter()
            .take(8)
            .map(|(_, name)| name)
            .collect::<Vec<_>>(),
    }));
    let report =
        commands::report_text(store, 7, None).unwrap_or_else(|_| "report unavailable".to_string());
    sections.push(json!({
        "title": "report · 7d",
        "lines": report.lines().take(18).collect::<Vec<_>>(),
    }));
    Ok(json!({
        "brand": "niuse",
        "title": "niuse · web",
        "sections": sections,
    }))
}

#[cfg(feature = "tui")]
async fn run_tui(cli: &Cli, store: &Store, replay: Option<PathBuf>) -> Result<()> {
    let settings = settings::resolve(cli, store)?;
    let cwd = config::resolve_cwd(cli.cwd.clone())?;
    let (policy, approval, _) =
        config::resolve_policy(&settings.sandbox, &settings.approval, &settings.network)?;
    let model = settings
        .model
        .clone()
        .unwrap_or_else(|| "deepseek-flash".to_string());
    if cli.computer != "off" && !niucode_provider::model_supports_images(&model) {
        eprintln!(
            "warning: computer observe sends screenshots; model {model} has no vision support"
        );
    }
    let thread = config::resolve_thread(cli.thread.clone(), store)?;
    let meta = driver::SessionMeta {
        cwd: &cwd,
        model: &model,
        provider: &settings.provider,
        policy,
        approval,
    };
    drop(driver::open_thread(&thread, &meta)?);

    let hooks =
        tui_hook::CompanionHooks::new_with_thread(store, thread.clone())?.with_cwd(cwd.clone());
    let options = niucode::TuiOptions {
        preset: settings.provider,
        model: Some(model),
        max_steps: settings.max_steps,
        no_thinking: cli.no_thinking,
        cwd: Some(cwd),
        session_dir: Some(store.root().to_path_buf()),
        resume: Some(thread),
        sandbox: settings.sandbox,
        approval: settings.approval,
        network: settings.network,
        replay,
        plan: cli.plan,
        computer: cli.computer.clone(),
        approval_keys: niucode::ApprovalKeys::custom('y', 'n', 's')
            .expect("distinct approval keys"),
        extra_tools: memory_tool::tools(store.root().to_path_buf()),
        allow_mcp_write: cli.allow_mcp_write,
        ..niucode::TuiOptions::default()
    };
    niucode::run_tui_with_hooks(options, Box::new(hooks)).await
}

#[cfg(test)]
mod docs_tests {
    use super::*;
    use clap::CommandFactory;
    use std::collections::HashSet;

    const README: &str = include_str!("../../../README.md");

    #[cfg(feature = "web")]
    #[test]
    fn web_new_thread_archives_and_recreates_the_active_thread() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().to_path_buf());
        store.init().unwrap();
        std::fs::write(store.session_path(), "").unwrap();
        let path = web_new_thread(&store, directory.path(), "m", "deepseek").unwrap();
        assert_eq!(path, store.session_path());
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("session_start"), "{text}");
        let archived = std::fs::read_dir(store.threads_dir()).unwrap().count();
        assert!(archived >= 1);
    }

    #[cfg(feature = "web")]
    #[test]
    fn web_switch_thread_moves_an_archived_chapter_back() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().to_path_buf());
        store.init().unwrap();
        let active = web_new_thread(&store, directory.path(), "m", "deepseek").unwrap();
        std::fs::create_dir_all(store.threads_dir()).unwrap();
        let archived = store.threads_dir().join("thread-1.jsonl");
        std::fs::copy(&active, &archived).unwrap();
        let expected = std::fs::read_to_string(&archived).unwrap();
        let switched =
            web_switch_thread(&store, "thread-1", directory.path(), "m", "deepseek").unwrap();
        assert_eq!(switched, active);
        assert_eq!(std::fs::read_to_string(&switched).unwrap(), expected);
    }

    #[cfg(feature = "web")]
    #[test]
    fn web_context_contains_product_sections() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::at(directory.path().to_path_buf());
        store.init().unwrap();
        let context = web_context(&store).unwrap();
        assert_eq!(context["brand"], "niuse");
        let sections = context["sections"].as_array().unwrap();
        assert!(sections.len() >= 9);
        assert!(sections
            .iter()
            .any(|section| section["title"] == "report · 7d"));
        assert!(sections.iter().any(|section| section["title"] == "threads"));
        assert!(sections.iter().any(|section| section["title"] == "thread"));
    }

    #[test]
    fn exec_parses_attachment_flags() {
        let cli = Cli::try_parse_from([
            "niuse",
            "exec",
            "look",
            "--attach",
            "note.md",
            "--image",
            "shot.png",
            "--computer",
            "observe",
        ])
        .expect("parse");
        assert_eq!(cli.computer, "observe");
        let Some(Command::Exec { attach, image, .. }) = cli.command else {
            panic!("expected exec");
        };
        assert_eq!(attach, [PathBuf::from("note.md")]);
        assert_eq!(image, [PathBuf::from("shot.png")]);
    }

    #[test]
    fn attachment_ingestion_enforces_kind() {
        let directory = tempfile::tempdir().unwrap();
        let store =
            niucode_attachments::AttachmentStore::open(directory.path().join("attachments"))
                .unwrap();
        let note = directory.path().join("note.txt");
        std::fs::write(&note, "hello").unwrap();
        let refs = repl::ingest_attachments(
            std::slice::from_ref(&note),
            Some(niucode_attachments::AttachmentKind::Text),
            &store,
        )
        .unwrap();
        assert_eq!(refs.len(), 1);
        assert!(repl::ingest_attachments(
            std::slice::from_ref(&note),
            Some(niucode_attachments::AttachmentKind::Image),
            &store
        )
        .is_err());
    }

    #[test]
    fn readme_cli_examples_name_real_subcommands() {
        let mut known: HashSet<String> = Cli::command()
            .get_subcommands()
            .map(|command| command.get_name().to_string())
            .collect();
        // `tui` is feature-gated; the docs are still correct for that build.
        known.insert("tui".to_string());
        let mut checked = 0;
        let mut fenced = false;
        for line in README.lines() {
            let line = line.trim();
            if line.starts_with("```") {
                fenced = !fenced;
                continue;
            }
            if !fenced {
                continue;
            }
            let Some(rest) = line.strip_prefix("niuse ") else {
                continue;
            };
            if rest.starts_with(char::is_whitespace) {
                continue;
            }
            let Some(first) = rest.split_whitespace().next() else {
                continue;
            };
            if first.starts_with('-') || first.starts_with('#') {
                continue;
            }
            assert!(
                known.contains(first),
                "README example `niuse {first}` is not a subcommand; known: {known:?}"
            );
            checked += 1;
        }
        assert!(
            checked >= 10,
            "expected the README command table, saw {checked}"
        );
    }

    #[test]
    fn readme_cli_flags_exist_on_the_documented_command() {
        let root = Cli::command();
        let mut checked = 0;
        let mut fenced = false;
        for line in README.lines() {
            let line = line.trim();
            if line.starts_with("```") {
                fenced = !fenced;
                continue;
            }
            if !fenced {
                continue;
            }
            let Some(rest) = line.strip_prefix("niuse ") else {
                continue;
            };
            if rest.starts_with(char::is_whitespace) {
                continue;
            }
            let tokens: Vec<&str> = rest.split_whitespace().collect();
            let Some(first) = tokens.first() else {
                continue;
            };
            if first.starts_with('-') || first.starts_with('#') || *first == "tui" {
                continue;
            }
            let Some(command) = root.find_subcommand(first).cloned() else {
                continue;
            };
            let mut commands = vec![command.clone()];
            commands.extend(command.get_subcommands().cloned());
            let mut valid: HashSet<String> = commands
                .iter()
                .flat_map(|command| command.get_arguments())
                .filter_map(|argument| argument.get_long().map(ToString::to_string))
                .collect();
            valid.extend(
                root.get_arguments()
                    .filter_map(|argument| argument.get_long().map(ToString::to_string)),
            );
            for flag in flags_in(line) {
                if flag == "help" || flag == "version" {
                    continue;
                }
                assert!(
                    valid.contains(&flag),
                    "README flag `--{flag}` is not valid for `niuse {first}`; valid: {valid:?}"
                );
                checked += 1;
            }
        }
        assert!(checked >= 5, "expected documented flags, saw {checked}");
    }

    fn flags_in(line: &str) -> Vec<String> {
        let mut flags = Vec::new();
        let mut chars = line.chars().peekable();
        while let Some(character) = chars.next() {
            if character != '-' || chars.peek() != Some(&'-') {
                continue;
            }
            chars.next();
            let mut flag = String::new();
            while let Some(next) = chars.peek().copied() {
                if next.is_ascii_alphanumeric() || next == '-' {
                    flag.push(next);
                    chars.next();
                } else {
                    break;
                }
            }
            if !flag.is_empty() {
                flags.push(flag);
            }
        }
        flags
    }

    #[test]
    fn readme_repl_commands_are_registered() {
        let line = README
            .lines()
            .find(|line| line.starts_with("Inside the REPL:"))
            .expect("README has an inside-the-REPL line");
        let mut checked = 0;
        for token in line.split('`') {
            let Some(command) = token.split_whitespace().next() else {
                continue;
            };
            if !command.starts_with('/') {
                continue;
            }
            assert!(
                repl::SLASH_COMMANDS.contains(&command),
                "README REPL command {command} is not registered"
            );
            checked += 1;
        }
        assert!(checked >= 10, "expected REPL commands, saw {checked}");
    }
}
