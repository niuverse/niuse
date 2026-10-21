//! Runtime settings: provider construction and the three security axes.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use niucode_agent::ApprovalMode;
use niucode_auth::AuthStore;
use niucode_provider::{OpenAiConfig, OpenAiProvider, Preset, Provider, ThinkingMode};
use niucode_tools::{NetworkMode, SandboxMode, SandboxPolicy};

pub struct ProviderBundle {
    pub provider: Arc<dyn Provider>,
    pub provider_id: String,
    pub model: String,
}

pub fn build_provider(
    preset: &str,
    model: Option<&str>,
    no_thinking: bool,
) -> Result<ProviderBundle> {
    let preset = Preset::parse(preset).with_context(|| format!("unknown provider {preset:?}"))?;
    let auth = AuthStore::new().context("initialize auth store")?;
    let credential = auth.resolve(preset.as_str())?.with_context(|| {
        format!(
            "no credential for {}; run `niucode auth set {}` or set {}",
            preset.as_str(),
            preset.as_str(),
            preset.api_key_env()
        )
    })?;

    let provider_id = preset.as_str().to_string();
    let model = model
        .map(str::to_string)
        .unwrap_or_else(|| default_model(&preset));

    let mut config = OpenAiConfig::preset_config(preset)?;
    config.api_key = credential.key.expose().to_string();
    config.model = model.clone();
    if no_thinking && preset == Preset::DeepSeek {
        config.thinking = Some(ThinkingMode::Disabled);
    }
    let provider: Arc<dyn Provider> = Arc::new(OpenAiProvider::for_preset(preset, config)?);
    Ok(ProviderBundle {
        provider,
        provider_id,
        model,
    })
}

fn default_model(preset: &Preset) -> String {
    niucode_provider::find_provider(preset.as_str())
        .map(|descriptor| descriptor.default_model.to_string())
        .unwrap_or_else(|| preset.default_model().to_string())
}

pub fn parse_sandbox(value: &str) -> Result<SandboxMode> {
    match value {
        "read-only" => Ok(SandboxMode::ReadOnly),
        "workspace-write" => Ok(SandboxMode::WorkspaceWrite),
        "full-access" => Ok(SandboxMode::FullAccess),
        other => bail!("unknown sandbox {other:?}; use read-only, workspace-write, or full-access"),
    }
}

pub fn parse_network(value: &str) -> Result<NetworkMode> {
    match value {
        "on" => Ok(NetworkMode::On),
        "off" => Ok(NetworkMode::Off),
        other => bail!("unknown network {other:?}; use on or off"),
    }
}

pub fn parse_approval(value: &str) -> Result<ApprovalMode> {
    match value {
        "ask" => Ok(ApprovalMode::Ask),
        "on-failure" => Ok(ApprovalMode::OnFailure),
        "never" => Ok(ApprovalMode::Never),
        other => bail!("unknown approval {other:?}; use ask, on-failure, or never"),
    }
}

/// Mirror niucode's fallback: without bwrap, `never` is refused and the other
/// modes degrade to explicit per-action approval.
pub fn resolve_policy(
    sandbox: &str,
    approval: &str,
    network: &str,
) -> Result<(SandboxPolicy, ApprovalMode, bool)> {
    let mut approval = parse_approval(approval)?;
    let policy = SandboxPolicy::new(parse_sandbox(sandbox)?, parse_network(network)?);
    let mut allow_unsandboxed = false;

    if policy.backend() == "unavailable" {
        match approval {
            ApprovalMode::Never => bail!(
                "sandbox backend bwrap is unavailable; install bubblewrap or use --approval ask"
            ),
            ApprovalMode::OnFailure => {
                eprintln!(
                    "warning: sandbox backend unavailable; falling back to per-action approval"
                );
                approval = ApprovalMode::Ask;
                allow_unsandboxed = true;
            }
            ApprovalMode::Ask => {
                eprintln!("warning: sandbox backend unavailable; using per-action approval");
                allow_unsandboxed = true;
            }
        }
    }

    Ok((policy, approval, allow_unsandboxed))
}

pub fn resolve_cwd(cwd: Option<PathBuf>) -> Result<PathBuf> {
    let cwd = match cwd {
        Some(path) => path,
        None => std::env::current_dir().context("resolve current directory")?,
    };
    let cwd = cwd
        .canonicalize()
        .with_context(|| format!("resolve working directory {}", cwd.display()))?;
    if !cwd.is_dir() {
        bail!("working directory {} is not a directory", cwd.display());
    }
    Ok(cwd)
}
