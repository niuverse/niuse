//! Session glue between niuse's stores and niucode's turn loop.

use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use niucode_agent::{
    project_history, run_with_attachments, run_with_history, AgentConfig, AgentEvent, ApprovalMode,
    ApprovalRequest, ComputerRuntime, RunOutcome,
};
use niucode_attachments::AttachmentStore;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use niucode_computer::ComputerPolicy;
use niucode_core::{load, now_ms, AttachmentRef, Event, SessionLog};
use niucode_mcp::{McpConfig, McpHost};
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

/// Build the platform computer runtime for a product surface. `off` is a
/// no-op; missing displays or TCC permissions warn and return `None`.
#[allow(clippy::needless_return)]
pub fn computer_runtime(policy: &str) -> Result<Option<Arc<std::sync::Mutex<ComputerRuntime>>>> {
    if policy == "off" {
        return Ok(None);
    }
    #[cfg(target_os = "linux")]
    {
        let policy: ComputerPolicy = policy
            .parse()
            .with_context(|| format!("invalid --computer policy {policy:?}"))?;
        match niucode_computer_x11::X11Backend::connect(None) {
            Ok(backend) => {
                let mut runtime = ComputerRuntime::new(Box::new(backend), policy);
                runtime.arm().context("arm computer use")?;
                eprintln!("computer: policy={} armed", policy.as_str());
                return Ok(Some(Arc::new(std::sync::Mutex::new(runtime))));
            }
            Err(error) => eprintln!("warning: computer unavailable: {error}"),
        }
    }
    #[cfg(target_os = "macos")]
    {
        let policy: ComputerPolicy = policy
            .parse()
            .with_context(|| format!("invalid --computer policy {policy:?}"))?;
        match niucode_computer_macos::MacBackend::connect(None) {
            Ok(backend) => {
                let mut runtime = ComputerRuntime::new(Box::new(backend), policy);
                runtime.arm().context("arm computer use")?;
                eprintln!(
                    "computer: policy={} armed (macOS TCC permissions required)",
                    policy.as_str()
                );
                return Ok(Some(Arc::new(std::sync::Mutex::new(runtime))));
            }
            Err(error) => eprintln!("warning: computer unavailable: {error}"),
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    eprintln!("warning: computer use is not supported in this build (got {policy:?})");
    Ok(None)
}

/// Run one user turn with attachments. The session history is rebuilt by the
/// agent's attachment-aware projection; `run_turn` remains the text-only path.
pub async fn run_turn_with_attachments(
    provider: &dyn Provider,
    log: &mut SessionLog,
    config: &AgentConfig,
    prompt: &str,
    attachments: Vec<AttachmentRef>,
    store: &AttachmentStore,
    renderer: &mut Renderer,
) -> Result<Option<RunOutcome>> {
    let mut observer = |event: AgentEvent| renderer.event(event);
    let mut approver = |request: &ApprovalRequest| approve(request);

    let future = run_with_attachments(
        provider,
        log,
        config,
        prompt,
        attachments,
        store,
        &mut approver,
        &mut observer,
    );
    tokio::pin!(future);

    tokio::select! {
        result = &mut future => result.map(Some),
        _ = tokio::signal::ctrl_c() => Ok(None),
    }
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

/// Mount MCP servers from `NIUSE_MCP_CONFIG` or `~/.niucode/mcp.toml`.
/// Read-only is the default; write-enabled servers need `allow_write` and an
/// interactive approval mode, and are filtered out otherwise. A broken
/// manifest disables MCP for this run instead of failing the session.
pub async fn mount_mcp(allow_write: bool, approval: ApprovalMode) -> Option<Arc<McpHost>> {
    let explicit = std::env::var_os("NIUSE_MCP_CONFIG").filter(|value| !value.is_empty());
    let path = match &explicit {
        Some(path) => PathBuf::from(path),
        None => crate::projects::default_root().ok()?.join("mcp.toml"),
    };
    if explicit.is_some() && !path.is_file() {
        eprintln!("warning: MCP config not found: {}", path.display());
        return None;
    }
    mount_mcp_from(&path, allow_write, approval).await
}

/// Mount from a specific manifest; callers that resolve the default path use
/// `mount_mcp`.
pub async fn mount_mcp_from(
    path: &Path,
    allow_write: bool,
    approval: ApprovalMode,
) -> Option<Arc<McpHost>> {
    let interactive = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    mount_mcp_from_with(path, allow_write, approval, interactive).await
}

async fn mount_mcp_from_with(
    path: &Path,
    allow_write: bool,
    approval: ApprovalMode,
    interactive: bool,
) -> Option<Arc<McpHost>> {
    let parsed = match McpConfig::load(path) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("warning: MCP config: {error:#}");
            return None;
        }
    };
    let write_interactive = allow_write && approval != ApprovalMode::Never && interactive;
    let all = match if allow_write {
        parsed.validated_with(true)
    } else {
        parsed.validated()
    } {
        Ok(servers) => servers,
        Err(error) => {
            eprintln!("warning: MCP disabled: {error:#}");
            return None;
        }
    };
    let mut skipped_write = false;
    let servers = if write_interactive {
        all
    } else {
        all.into_iter()
            .filter(|server| {
                if server.read_only {
                    true
                } else {
                    skipped_write = true;
                    false
                }
            })
            .collect()
    };
    if skipped_write {
        eprintln!(
            "warning: MCP write servers need --allow-mcp-write, an interactive approval mode, and a terminal; read-only servers only"
        );
    }
    if servers.is_empty() {
        return None;
    }
    let mount_writes = write_interactive && servers.iter().any(|server| !server.read_only);
    match McpHost::mount_with(&servers, Duration::from_secs(10), mount_writes).await {
        Ok(host) => {
            for mount in host.mounts() {
                eprintln!("mcp: {} · {} tool(s)", mount.server, mount.tools.len());
            }
            Some(Arc::new(host))
        }
        Err(error) => {
            eprintln!("warning: MCP disabled: {error:#}");
            None
        }
    }
}

/// Append one `mcp_mount` event per server. The hash covers command, args,
/// read_only, write_tools, and env keys, never env values.
pub fn record_mcp_mounts(log: &SessionLog, host: &McpHost) -> Result<()> {
    for mount in host.mounts() {
        log.append(&Event::McpMount {
            server: mount.server,
            config_hash: mount.config_hash,
            write_tools: mount.write_tools,
            ts: now_ms(),
        })?;
    }
    Ok(())
}

/// Narrow an agent config to read-only planning tools. Turning plan mode off
/// leaves any explicit tool allowlist in place.
pub fn apply_plan_mode(config: &mut AgentConfig, plan: bool) {
    config.plan_mode = plan;
    if plan {
        config.allowed_tools = Some(
            ["read", "grep", "glob"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mounts_a_read_only_mcp_server_from_a_manifest() {
        let directory = tempfile::tempdir().unwrap();
        let config = directory.path().join("mcp.toml");
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../niucode/crates/niucode-mcp/tests/fixture_server.py");
        std::fs::write(
            &config,
            format!(
                "[[server]]\nname = \"fixture\"\ncommand = \"python3\"\nargs = [\"{}\"]\nread_only = true\n\n[server.env]\nPATH = \"{}\"\n",
                fixture.display(),
                std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".into())
            ),
        )
        .unwrap();
        let host = mount_mcp_from(&config, false, ApprovalMode::Ask)
            .await
            .expect("mount");
        let names: Vec<String> = host
            .specs()
            .into_iter()
            .map(|spec| spec.function.name)
            .collect();
        assert!(names.contains(&"mcp__fixture__echo".to_string()));
        let call = host
            .call(
                "mcp__fixture__echo",
                serde_json::json!({"text": "from niuse"}),
                Duration::from_secs(5),
            )
            .await
            .expect("call");
        assert_eq!(call.output, "from niuse");
        host.shutdown().await.unwrap();
    }

    #[test]
    fn plan_mode_narrows_tools_and_does_not_widen_on_exit() {
        let mut config = AgentConfig::new("test", ".");
        apply_plan_mode(&mut config, true);
        assert!(config.plan_mode);
        assert_eq!(
            config.allowed_tools.as_ref().unwrap(),
            &["read".to_string(), "grep".to_string(), "glob".to_string()]
        );
        apply_plan_mode(&mut config, false);
        assert!(!config.plan_mode);
        assert_eq!(config.allowed_tools.as_ref().unwrap().len(), 3);
    }

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

    #[tokio::test]
    async fn write_mount_needs_the_flag_and_an_interactive_terminal() {
        let directory = tempfile::tempdir().unwrap();
        let config = directory.path().join("mcp.toml");
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../niucode/crates/niucode-mcp/tests/fixture_server.py");
        std::fs::write(
            &config,
            format!(
                "[[server]]\nname = \"fixture\"\ncommand = \"python3\"\nargs = [\"{}\"]\nread_only = false\nwrite_tools = [\"echo\"]\n\n[server.env]\nPATH = \"{}\"\n",
                fixture.display(),
                std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".into())
            ),
        )
        .unwrap();
        assert!(mount_mcp_from_with(&config, false, ApprovalMode::Ask, true)
            .await
            .is_none());
        assert!(
            mount_mcp_from_with(&config, true, ApprovalMode::Never, true)
                .await
                .is_none()
        );
        assert!(mount_mcp_from_with(&config, true, ApprovalMode::Ask, false)
            .await
            .is_none());
        let host = mount_mcp_from_with(&config, true, ApprovalMode::Ask, true)
            .await
            .expect("write mount");
        assert!(host.is_mutating("mcp__fixture__echo"));
        host.shutdown().await.unwrap();
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
