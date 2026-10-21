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
    /// Show or search the journal
    Journal {
        #[arg(long, default_value_t = 7)]
        days: usize,
        /// Search entries instead of listing recent days
        #[arg(long)]
        search: Option<String>,
    },
    /// Create or show identity.md
    Identity {
        #[command(subcommand)]
        command: IdentityCommand,
    },
    /// Mine recent niucode sessions for memory candidates
    Bootstrap {
        /// Maximum number of sessions to scan
        #[arg(long, default_value_t = 40)]
        limit: usize,
        /// Only sessions modified within this many days
        #[arg(long, default_value_t = 90)]
        since_days: i64,
    },
    /// Review bootstrapped candidates
    Review,
    /// Check local setup without calling a model
    Doctor,
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
    },
    /// Add a memory directly
    Add {
        text: String,
        /// preference or lesson
        #[arg(long, default_value = "preference")]
        kind: String,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        /// Store even if the secret scan matches
        #[arg(long)]
        force: bool,
    },
    /// Search memories, including expired ones
    Search {
        query: String,
        /// Exclude expired entries
        #[arg(long)]
        fresh: bool,
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
}

#[tokio::main]
async fn main() -> Result<()> {
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
            MemCommand::List { all } => commands::mem_list(&store, all),
            MemCommand::Add {
                text,
                kind,
                tags,
                force,
            } => commands::mem_add(&store, &text, &kind, tags, force),
            MemCommand::Search { query, fresh } => commands::mem_search(&store, &query, fresh),
            MemCommand::Show { id } => commands::mem_show(&store, &id),
            MemCommand::Edit { id } => commands::mem_edit(&store, &id),
            MemCommand::Forget { id } => commands::mem_forget(&store, &id),
            MemCommand::Confirm { id } => commands::mem_confirm(&store, &id),
            MemCommand::Review => commands::mem_review(&store),
        },
        Some(Command::Loops { command }) => match command {
            LoopsCommand::List { all } => commands::loops_list(&store, all),
            LoopsCommand::Add { text, due } => commands::loops_add(&store, &text, due.as_deref()),
            LoopsCommand::Done { id } => commands::loops_done(&store, &id),
        },
        Some(Command::Export { out, session }) => {
            commands::export(&store, out.as_deref(), session.as_deref())
        }
        Some(Command::Journal { days, search }) => match search {
            Some(query) => commands::journal_search(&store, &query),
            None => commands::journal_show(&store, days),
        },
        Some(Command::Identity { command }) => match command {
            IdentityCommand::Init { force } => commands::identity_init(&store, force),
            IdentityCommand::Show => commands::identity_show(&store),
        },
        Some(Command::Bootstrap { limit, since_days }) => {
            let settings = settings::resolve(&cli, &store)?;
            let app = repl::App::new(&cli, &settings, store)?;
            bootstrap::run(
                &app.store,
                app.provider.as_ref(),
                &app.model,
                limit,
                since_days,
            )
            .await
        }
        Some(Command::Review) => bootstrap::review(&store),
        Some(Command::Doctor) => {
            let settings = settings::resolve(&cli, &store)?;
            commands::doctor(&store, &settings.provider)
        }
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
        Some(Command::Stats { days }) => commands::stats(&store, days),
        Some(Command::Status) => commands::status(&store),
    }
}
