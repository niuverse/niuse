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
        create_private_file(path).with_context(|| format!("create thread {}", path.display()))?;
    }
    set_private(path)?;

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

/// Move the current thread into the archive directory. Returns `None` when
/// there is nothing to archive.
pub fn archive_thread(path: &Path, directory: &Path) -> Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }
    fs::create_dir_all(directory)
        .with_context(|| format!("create directory {}", directory.display()))?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or(0);
    let destination = unique_archive_path(directory, stamp);
    fs::rename(path, &destination)
        .with_context(|| format!("archive {} to {}", path.display(), destination.display()))?;
    Ok(Some(destination))
}

fn unique_archive_path(directory: &Path, stamp: u128) -> PathBuf {
    let mut destination = directory.join(format!("thread-{stamp}.jsonl"));
    let mut suffix = 1;
    while destination.exists() {
        destination = directory.join(format!("thread-{stamp}-{suffix}.jsonl"));
        suffix += 1;
    }
    destination
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
fn create_private_file(path: &Path) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)?;
    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn thread_files_are_created_private() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("thread.jsonl");
        create_private_file(&path).unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn existing_thread_permissions_are_repaired() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("thread.jsonl");
        fs::write(&path, "{}\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        set_private(&path).unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn archive_moves_the_thread() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("thread.jsonl");
        fs::write(&source, "{}\n").unwrap();
        let archive = directory.path().join("threads");
        let destination = archive_thread(&source, &archive).unwrap().unwrap();
        assert!(!source.exists());
        assert!(destination.exists());
        assert_eq!(fs::read_to_string(&destination).unwrap(), "{}\n");
    }

    #[test]
    fn archive_names_do_not_collide_within_one_millisecond() {
        let directory = tempfile::tempdir().unwrap();
        let archive = directory.path().join("threads");
        fs::create_dir_all(&archive).unwrap();
        let first = unique_archive_path(&archive, 42);
        fs::write(&first, "{}\n").unwrap();
        let second = unique_archive_path(&archive, 42);
        assert_ne!(first, second);
        assert!(second.to_string_lossy().contains("thread-42-1"));
    }

    #[test]
    fn archiving_a_missing_thread_is_a_noop() {
        let directory = tempfile::tempdir().unwrap();
        assert!(archive_thread(
            &directory.path().join("missing.jsonl"),
            &directory.path().join("threads")
        )
        .unwrap()
        .is_none());
    }
}
