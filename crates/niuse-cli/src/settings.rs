//! Defaults from `~/.niuse/config.toml`.
//!
//! Precedence is CLI flag → config file → built-in. The file is optional; an
//! absent file is not an error and an unreadable one is.

use anyhow::{Context, Result};
use niuse_core::Store;
use serde::Deserialize;

use crate::Cli;

pub const TEMPLATE: &str = "\
# niuse defaults. CLI flags override these; built-ins are the fallback.
[defaults]
# provider = \"deepseek\"
# model = \"deepseek-flash\"
# sandbox = \"workspace-write\"   # read-only | workspace-write | full-access
# approval = \"on-failure\"       # ask | on-failure | never
# network = \"off\"               # on | off
# max_steps = 0                 # 0 = no hard cap
";

#[derive(Debug, Default, Deserialize)]
pub struct FileConfig {
    #[serde(default)]
    pub defaults: Defaults,
}

#[derive(Debug, Default, Deserialize)]
pub struct Defaults {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub sandbox: Option<String>,
    pub approval: Option<String>,
    pub network: Option<String>,
    pub max_steps: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub provider: String,
    pub model: Option<String>,
    pub sandbox: String,
    pub approval: String,
    pub network: String,
    pub max_steps: usize,
}

pub fn load(store: &Store) -> Result<FileConfig> {
    match store.read_config()? {
        Some(text) => toml::from_str(&text)
            .with_context(|| format!("parse {}", store.config_path().display())),
        None => Ok(FileConfig::default()),
    }
}

pub fn resolve(cli: &Cli, store: &Store) -> Result<Settings> {
    let file = load(store)?;
    Ok(Settings {
        provider: pick(
            cli.preset.clone(),
            file.defaults.provider,
            "deepseek".to_string(),
        ),
        model: cli.model.clone().or(file.defaults.model),
        sandbox: pick(
            cli.sandbox.clone(),
            file.defaults.sandbox,
            "workspace-write".to_string(),
        ),
        approval: pick(
            cli.approval.clone(),
            file.defaults.approval,
            "on-failure".to_string(),
        ),
        network: pick(
            cli.network.clone(),
            file.defaults.network,
            "off".to_string(),
        ),
        max_steps: cli.max_steps.or(file.defaults.max_steps).unwrap_or(0),
    })
}

fn pick(cli: Option<String>, file: Option<String>, built_in: String) -> String {
    cli.or(file).unwrap_or(built_in)
}

pub fn init(store: &Store, force: bool) -> Result<bool> {
    let path = store.config_path();
    if path.exists() && !force {
        return Ok(false);
    }
    store.write_config(TEMPLATE)?;
    Ok(true)
}

pub fn show(store: &Store) -> Result<()> {
    let path = store.config_path();
    println!("{}", path.display());
    match store.read_config()? {
        Some(text) => print!("{text}"),
        None => println!("(missing; using built-in defaults)"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_defaults() {
        let file: FileConfig = toml::from_str(
            r#"
[defaults]
provider = "openai"
sandbox = "read-only"
max_steps = 4
"#,
        )
        .unwrap();
        assert_eq!(file.defaults.provider.as_deref(), Some("openai"));
        assert_eq!(file.defaults.sandbox.as_deref(), Some("read-only"));
        assert_eq!(file.defaults.max_steps, Some(4));
        assert_eq!(file.defaults.approval, None);
    }

    #[test]
    fn cli_beats_file_beats_builtin() {
        assert_eq!(
            pick(Some("cli".into()), Some("file".into()), "builtin".into()),
            "cli"
        );
        assert_eq!(pick(None, Some("file".into()), "builtin".into()), "file");
        assert_eq!(pick(None, None, "builtin".into()), "builtin");
    }

    #[test]
    fn empty_file_is_valid() {
        let file: FileConfig = toml::from_str("").unwrap();
        assert_eq!(file.defaults.provider, None);
    }
}
