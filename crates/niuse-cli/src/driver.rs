//! Session glue between niuse's stores and niucode's turn loop.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use niucode_agent::{
    project_history, run_with_history, AgentConfig, AgentEvent, ApprovalMode, ApprovalRequest,
    RunOutcome,
};
use niucode_core::{load, now_ms, Event, SessionLog};
use niucode_provider::{ChatMessage, Provider};
use niucode_tools::SandboxPolicy;

use crate::render::{approve, Renderer};

pub struct SessionMeta<'a> {
    pub cwd: &'a Path,
    pub model: &'a str,
    pub provider: &'a str,
    pub policy: SandboxPolicy,
    pub approval: ApprovalMode,
}

pub struct Thread {
    pub log: SessionLog,
    pub path: PathBuf,
}

/// Open the durable thread, creating it on first use.
pub fn open_thread(path: &Path, meta: &SessionMeta<'_>) -> Result<Thread> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create directory {}", parent.display()))?;
    }
    let created = !path.exists() || fs::metadata(path).map(|m| m.len() == 0).unwrap_or(true);
    if !path.exists() {
        fs::File::create(path).with_context(|| format!("create thread {}", path.display()))?;
        set_private(path)?;
    }

    let log = SessionLog::open(path)?;
    if created {
        append_start(&log, meta)?;
    }
    Ok(Thread {
        log,
        path: path.to_path_buf(),
    })
}

pub fn append_start(log: &SessionLog, meta: &SessionMeta<'_>) -> Result<()> {
    log.append(&Event::SessionStart {
        id: log.id().to_string(),
        cwd: meta.cwd.display().to_string(),
        model: meta.model.to_string(),
        provider: meta.provider.to_string(),
        ts: now_ms(),
    })?;
    log.append(&Event::Policy {
        sandbox: meta.policy.mode.as_str().to_string(),
        approval: meta.approval.as_str().to_string(),
        network: meta.policy.network.as_str().to_string(),
        backend: meta.policy.backend().to_string(),
        ts: now_ms(),
    })?;
    Ok(())
}

pub fn append_end(log: &SessionLog) -> Result<()> {
    log.append(&Event::SessionEnd { ts: now_ms() })
}

pub fn history(path: &Path) -> Result<Vec<ChatMessage>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let loaded = load(path).with_context(|| format!("load session {}", path.display()))?;
    for warning in loaded.warnings {
        eprintln!("session warning: {warning}");
    }
    project_history(&loaded.events)
}

pub fn new_session(directory: &Path) -> Result<SessionLog> {
    fs::create_dir_all(directory)
        .with_context(|| format!("create directory {}", directory.display()))?;
    let log = SessionLog::create(directory)?;
    set_private(log.path())?;
    Ok(log)
}

#[allow(clippy::too_many_arguments)]
pub fn agent_config(
    provider: &str,
    model: &str,
    cwd: &Path,
    system_prompt: String,
    policy: SandboxPolicy,
    approval: ApprovalMode,
    allow_unsandboxed: bool,
    max_steps: usize,
) -> AgentConfig {
    let mut config = AgentConfig::new(model, cwd);
    config.provider = provider.to_string();
    config.system_prompt = system_prompt;
    config.sandbox = policy;
    config.approval = approval;
    config.allow_unsandboxed = allow_unsandboxed;
    config.max_steps = max_steps;
    let budget = niucode_provider::derived_context_budget(provider, model);
    if budget > 0 {
        config.context.budget_tokens = budget;
    }
    config
}

/// Run one user turn. `Ok(None)` means the user cancelled with Ctrl-C.
pub async fn run_turn(
    provider: &dyn Provider,
    log: &mut SessionLog,
    config: &AgentConfig,
    prompt: &str,
    history: Vec<ChatMessage>,
    renderer: &mut Renderer,
) -> Result<Option<RunOutcome>> {
    let mut observer = |event: AgentEvent| renderer.event(event);
    let mut approver = |request: &ApprovalRequest| approve(request);

    let future = run_with_history(
        provider,
        log,
        config,
        prompt,
        history,
        &mut approver,
        &mut observer,
    );
    tokio::pin!(future);

    tokio::select! {
        result = &mut future => result.map(Some),
        _ = tokio::signal::ctrl_c() => Ok(None),
    }
}

#[cfg(unix)]
fn set_private(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_private(_path: &Path) -> Result<()> {
    Ok(())
}
