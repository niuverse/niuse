//! Read-only view over niucode's per-project workspace metadata.
//!
//! niucode owns this layout. niuse only reads `projects/*/project.toml` and the
//! session filenames; it never creates, repairs, or rewrites anything here.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result};
use serde::Deserialize;

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
struct ProjectMeta {
    #[serde(default)]
    schema_version: u32,
    cwd: String,
    #[serde(default)]
    last_seen: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub key: String,
    pub cwd: PathBuf,
    pub last_seen: u64,
    pub sessions: usize,
    pub latest_session: Option<u64>,
    pub missing: bool,
}

impl Project {
    pub fn name(&self) -> String {
        self.cwd
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.key.clone())
    }

    /// Metadata and session mtimes can disagree; the newer one wins.
    pub fn activity(&self) -> u64 {
        self.last_seen.max(self.latest_session.unwrap_or(0))
    }
}

#[derive(Debug, Default)]
pub struct Registry {
    pub projects: Vec<Project>,
    pub warnings: Vec<String>,
}

impl Registry {
    pub fn discover(root: &Path) -> Self {
        let mut registry = Self::default();
        let projects_dir = root.join("projects");
        let entries = match fs::read_dir(&projects_dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return registry,
            Err(error) => {
                registry
                    .warnings
                    .push(format!("read {}: {error}", projects_dir.display()));
                return registry;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    registry.warnings.push(format!(
                        "read an entry in {}: {error}",
                        projects_dir.display()
                    ));
                    continue;
                }
            };
            let path = entry.path();
            if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                continue;
            }
            let key = entry.file_name().to_string_lossy().into_owned();
            let meta_path = path.join("project.toml");
            let text = match fs::read_to_string(&meta_path) {
                Ok(text) => text,
                Err(error) => {
                    registry
                        .warnings
                        .push(format!("skip {key}: cannot read project.toml: {error}"));
                    continue;
                }
            };
            let meta: ProjectMeta = match toml::from_str(&text) {
                Ok(meta) => meta,
                Err(error) => {
                    registry
                        .warnings
                        .push(format!("skip {key}: malformed project.toml: {error}"));
                    continue;
                }
            };
            if meta.schema_version != SCHEMA_VERSION {
                registry.warnings.push(format!(
                    "skip {key}: unsupported project schema {}",
                    meta.schema_version
                ));
                continue;
            }
            let cwd = PathBuf::from(&meta.cwd);
            if !cwd.is_absolute() {
                registry
                    .warnings
                    .push(format!("skip {key}: cwd is not absolute"));
                continue;
            }
            let (sessions, latest_session) = session_stats(&path.join("sessions"));
            registry.projects.push(Project {
                key,
                missing: !cwd.is_dir(),
                cwd,
                last_seen: meta.last_seen,
                sessions,
                latest_session,
            });
        }

        registry.projects.sort_by(|left, right| {
            right
                .activity()
                .cmp(&left.activity())
                .then_with(|| left.key.cmp(&right.key))
        });
        registry
    }

    pub fn current(&self, cwd: &Path) -> Option<&Project> {
        let cwd = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
        self.projects.iter().find(|project| {
            if project.cwd == cwd {
                return true;
            }
            project
                .cwd
                .canonicalize()
                .map(|path| path == cwd)
                .unwrap_or(false)
        })
    }
}

/// `NIUCODE_HOME` wins, then `$HOME/.niucode`, matching niucode's resolver.
pub fn default_root() -> Result<PathBuf> {
    if let Some(home) = env::var_os("NIUCODE_HOME") {
        if !home.is_empty() {
            return Ok(PathBuf::from(home));
        }
    }
    let home = env::var_os("HOME").context("HOME is not set; set NIUCODE_HOME")?;
    Ok(PathBuf::from(home).join(".niucode"))
}

/// A compact local-calendar label for a millisecond timestamp.
pub fn day_label(millis: u64) -> String {
    if millis == 0 {
        return "-".to_string();
    }
    let date = niuse_core::Date::from_millis_local(millis);
    match date.age_days(niuse_core::Date::today()) {
        0 => "今天".to_string(),
        1 => "昨天".to_string(),
        days => format!("{days} 天前"),
    }
}

fn session_stats(directory: &Path) -> (usize, Option<u64>) {
    let mut sessions = 0;
    let mut latest: Option<u64> = None;
    let Ok(entries) = fs::read_dir(directory) else {
        return (0, None);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("jsonl") {
            continue;
        }
        sessions += 1;
        if let Ok(modified) = entry.metadata().and_then(|metadata| metadata.modified()) {
            if let Ok(age) = modified.duration_since(UNIX_EPOCH) {
                let millis = age.as_millis() as u64;
                latest = Some(latest.map_or(millis, |value| value.max(millis)));
            }
        }
    }
    (sessions, latest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_project(root: &Path, key: &str, cwd: &Path, last_seen: u64, sessions: usize) {
        let directory = root.join("projects").join(key);
        fs::create_dir_all(directory.join("sessions")).unwrap();
        fs::write(
            directory.join("project.toml"),
            format!(
                "schema_version = 1\ncwd = \"{}\"\nlast_seen = {last_seen}\n",
                cwd.display()
            ),
        )
        .unwrap();
        for index in 0..sessions {
            fs::write(
                directory.join("sessions").join(format!("s{index}.jsonl")),
                "{}\n",
            )
            .unwrap();
        }
    }

    #[test]
    fn discovers_sorts_and_counts() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let older = root.join("older");
        let newer = root.join("newer");
        fs::create_dir_all(&older).unwrap();
        fs::create_dir_all(&newer).unwrap();
        write_project(root, "older-key", &older, 10, 1);
        write_project(root, "newer-key", &newer, 20, 3);

        let registry = Registry::discover(root);
        assert!(registry.warnings.is_empty());
        assert_eq!(registry.projects.len(), 2);
        assert_eq!(registry.projects[0].key, "newer-key");
        assert_eq!(registry.projects[0].sessions, 3);
        assert_eq!(registry.projects[1].key, "older-key");
        assert!(!registry.projects[0].missing);
        assert_eq!(registry.current(&newer).unwrap().key, "newer-key");
    }

    #[test]
    fn missing_cwd_is_flagged_but_kept() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        write_project(root, "gone", &root.join("does-not-exist"), 10, 0);
        let registry = Registry::discover(root);
        assert_eq!(registry.projects.len(), 1);
        assert!(registry.projects[0].missing);
    }

    #[test]
    fn corrupt_entries_are_skipped_with_a_warning() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let broken = root.join("projects").join("broken");
        fs::create_dir_all(&broken).unwrap();
        fs::write(broken.join("project.toml"), "not toml =").unwrap();
        write_project(root, "good", root, 10, 0);

        let registry = Registry::discover(root);
        assert_eq!(registry.projects.len(), 1);
        assert_eq!(registry.projects[0].key, "good");
        assert_eq!(registry.warnings.len(), 1);
        assert!(registry.warnings[0].contains("broken"));
    }

    #[test]
    fn missing_root_is_empty() {
        let directory = tempfile::tempdir().unwrap();
        let registry = Registry::discover(&directory.path().join("absent"));
        assert!(registry.projects.is_empty());
        assert!(registry.warnings.is_empty());
    }
}
