//! `niuse projects`: a read-only view over niucode's project registry.

use std::path::Path;

use anyhow::Result;

use crate::projects::{day_label, default_root, Registry};

pub fn projects(limit: usize, all: bool, json: bool) -> Result<()> {
    let root = default_root()?;
    let registry = Registry::discover(&root);
    for warning in &registry.warnings {
        eprintln!("warning: {warning}");
    }

    let registered = registry.projects.len();
    let matched: Vec<_> = registry
        .projects
        .iter()
        .filter(|project| all || (!project.missing && !is_temporary(&project.cwd)))
        .collect();
    let total = matched.len();
    let shown: Vec<_> = matched.into_iter().take(limit).collect();

    if json {
        let payload = serde_json::json!({
            "root": root.display().to_string(),
            "all": registered,
            "total": total,
            "shown": shown.len(),
            "projects": shown.iter().map(|project| serde_json::json!({
                "key": project.key,
                "name": project.name(),
                "cwd": project.cwd.display().to_string(),
                "last_seen": project.last_seen,
                "activity": project.activity(),
                "last_seen_day": day_label(project.activity()),
                "sessions": project.sessions,
                "missing": project.missing,
            })).collect::<Vec<_>>(),
            "warnings": registry.warnings.clone(),
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    if shown.is_empty() {
        if registered == 0 {
            println!("没有找到 niucode 项目：{}", root.display());
        } else {
            println!(
                "没有活跃项目；--all 查看全部 {} 条登记：{}",
                registered,
                root.display()
            );
        }
        return Ok(());
    }
    println!("{} 个项目（共 {} 条登记；最近活跃优先）", total, registered);
    for project in shown {
        println!(
            "{:<22} {:<10} {:>4} sessions  {}{}",
            project.name(),
            day_label(project.activity()),
            project.sessions,
            project.cwd.display(),
            if project.missing {
                "（目录不存在）"
            } else {
                ""
            }
        );
    }
    Ok(())
}

/// Temporary checkouts from tests and one-off runs are noise in the default
/// view; they are still reachable with `--all`.
fn is_temporary(cwd: &Path) -> bool {
    let temp = std::env::temp_dir();
    let temp = temp.canonicalize().unwrap_or(temp);
    let cwd = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    cwd.starts_with(temp) || cwd.starts_with("/tmp") || cwd.starts_with("/var/folders")
}
