mod bootstrap;
mod commands;
mod config;
mod driver;
mod export;
mod render;
mod repl;
mod settings;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use niuse_core::Store;

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
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Run one prompt through the agent, then exit
    Exec { prompt: String },
    /// Inspect and edit durable memory
    Mem {
        #[command(subcommand)]
        command: MemCommand,
    },
    /// Inspect and close open loops
    Loops {
        #[command(subcommand)]
        command: LoopsCommand,
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
    /// Check local setup without calling a model
    Doctor,
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
        /// Window in days; 0 means all time
        #[arg(long, default_value_t = 7)]
        days: u64,
        /// Add a per-day breakdown
        #[arg(long)]
        trend: bool,
        /// Emit JSON (summary plus per-day rows)
        #[arg(long)]
        json: bool,
    },
    /// Show store and prompt status
    Status,
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
    },
    /// Show one memory in full
    Show { id: String },
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
enum LoopsCommand {
    /// List loops
    List {
        /// Include closed loops
        #[arg(long)]
        all: bool,
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
    Show,
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
    Show,
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
    let store = match &cli.home {
        Some(path) => Store::at(path.clone()),
        None => Store::discover()?,
    };
    store.init()?;
    let command = cli.command.clone();

    match command {
        None => {
            let settings = settings::resolve(&cli, &store)?;
            let app = repl::App::new(&cli, &settings, store)?;
            app.run().await
        }
        Some(Command::Exec { prompt }) => {
            let settings = settings::resolve(&cli, &store)?;
            let app = repl::App::new(&cli, &settings, store)?;
            repl::run_exec(app, prompt).await
        }
        Some(Command::Mem { command }) => match command {
            MemCommand::List { all, json, kind } => {
                commands::mem_list(&store, all, json, kind.as_deref())
            }
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
            } => commands::mem_search(&store, &query, fresh, json, kind.as_deref()),
            MemCommand::Show { id } => commands::mem_show(&store, &id),
            MemCommand::Edit { id } => commands::mem_edit(&store, &id),
            MemCommand::Forget { id } => commands::mem_forget(&store, &id),
            MemCommand::Confirm { id } => commands::mem_confirm(&store, &id),
            MemCommand::Review => commands::mem_review(&store),
            MemCommand::Prune { apply } => commands::mem_prune(&store, apply),
        },
        Some(Command::Loops { command }) => match command {
            LoopsCommand::List { all } => commands::loops_list(&store, all),
            LoopsCommand::Add { text, due } => commands::loops_add(&store, &text, due.as_deref()),
            LoopsCommand::Done { id } => commands::loops_done(&store, &id),
        },
        Some(Command::Threads { json }) => {
            let active = cli.thread.clone().unwrap_or_else(|| store.session_path());
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
            IdentityCommand::Show => commands::identity_show(&store),
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
        Some(Command::Doctor) => {
            let settings = settings::resolve(&cli, &store)?;
            commands::doctor(&store, &settings.provider)
        }
        Some(Command::User { command }) => match command {
            UserCommand::Show => commands::user_show(&store),
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
        },
        Some(Command::Stats { days, trend, json }) => commands::stats(&store, days, trend, json),
        Some(Command::Status) => {
            let thread = cli.thread.clone().unwrap_or_else(|| store.session_path());
            commands::status(&store, &thread)
        }
    }
}
