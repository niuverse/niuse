//! Cold start: read old niucode sessions, extract durable candidates with one
//! cheap model call per session, and queue them for review.
//!
//! Tool output is deliberately dropped before extraction: the value is in what
//! was said and decided, not in the file contents the agent already read.

use std::cmp::Reverse;
use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use niucode_core::{load, Event};
use niucode_provider::{ChatMessage, CompletionRequest, Provider, StreamEvent};
use niuse_core::model::{normalize_tags, one_line};
use niuse_core::policy::{conflicts, secret_reason};
use niuse_core::{Date, MemoryEntry, MemoryKind, Store};
use serde::Deserialize;

const EXTRACTION_SYSTEM: &str = "你是记忆抽取器。下面是一段 coding agent 与用户的真实历史对话。\n\
只抽取三类跨会话仍然有用的信息：用户的偏好、做过的决策、踩过的坑。\n\
每条必须同时满足：脱离这段对话也能看懂；在仓库里 grep 不出来；一句话说得清。\n\
不要抽取：代码事实、文件路径、环境配置、临时任务进度、别人的信息、任何密钥。\n\
输出 JSON 数组，元素形如 {\"kind\":\"preference|lesson\",\"text\":\"...\",\"tags\":[\"...\"]}。\n\
没有值得记的就输出 []。不要输出 JSON 以外的任何文字。";

const PROFILE_SYSTEM: &str =
    "你是用户画像整理器。根据下面的历史对话，写一份 user.md 草稿，用 Markdown，不超过 20 行。\n\
包含三个小节：## 我在做什么、## 我的工作方式、## 我的雷区。\n\
只写对话中有证据的内容；不确定的不要编。不要写代码、文件路径、密钥、别人的信息。\n\
只输出 Markdown，不要解释。";

const MAX_MESSAGE_CHARS: usize = 2_000;
const MAX_PROFILE_CHARS: usize = 10_000;
const MAX_TRANSCRIPT_CHARS: usize = 12_000;

#[derive(Debug, Deserialize)]
struct Candidate {
    kind: String,
    text: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Clone)]
struct Pending {
    kind: MemoryKind,
    tags: Vec<String>,
    text: String,
    source: String,
}

pub struct BootstrapOptions {
    pub limit: usize,
    pub since_days: i64,
    pub dry_run: bool,
    pub profile: bool,
    pub force_profile: bool,
    pub profile_only: bool,
}

pub async fn run(
    store: &Store,
    provider: &dyn Provider,
    model: &str,
    options: BootstrapOptions,
) -> Result<()> {
    let BootstrapOptions {
        limit,
        since_days,
        dry_run,
        profile,
        force_profile,
        profile_only,
    } = options;
    let profile = profile || profile_only;
    let candidates = recent_sessions(usize::MAX, since_days)?;
    if candidates.is_empty() {
        println!("没有找到最近 {since_days} 天内的 niucode session。");
        return Ok(());
    }

    // An empty or tool-only session is not worth a model call. Fill the budget
    // with sessions that actually contain user or assistant prose.
    let mut sessions: Vec<(PathBuf, String)> = Vec::new();
    for path in candidates {
        if let Some(transcript) = transcript(&path)? {
            sessions.push((path, transcript));
            if sessions.len() >= limit {
                break;
            }
        }
    }
    if sessions.is_empty() {
        println!("最近 {since_days} 天内的 session 都没有可抽取的对话。");
        return Ok(());
    }
    println!(
        "扫描 {} 个 session（最近 {} 天，上限 {}）。",
        sessions.len(),
        since_days,
        limit
    );

    if dry_run {
        println!("dry run：不会调用模型。");
        for (path, transcript) in &sessions {
            let name = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("session");
            println!("  {name}  {} chars", transcript.chars().count());
        }
        if profile {
            println!("would draft user.md from the same sessions");
        }
        return Ok(());
    }

    if profile {
        match store.user()? {
            Some(text) if !text.trim().is_empty() && !force_profile => {
                println!("user.md 已存在；跳过 profile（用 --force-profile 覆盖）");
            }
            _ => write_profile(store, provider, model, &sessions).await?,
        }
    }
    if profile_only {
        println!("profile-only：跳过记忆候选。");
        return Ok(());
    }

    let mut raw: Vec<Pending> = Vec::new();
    let mut skipped = 0usize;
    let progress = io::stderr().is_terminal();
    for (index, (path, transcript)) in sessions.iter().enumerate() {
        if progress {
            eprint!("\r[{}/{}] 抽取中…", index + 1, sessions.len());
            let _ = io::stderr().flush();
        }

        match extract(provider, model, transcript).await {
            Ok(candidates) => {
                let source = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("session")
                    .to_string();
                for candidate in candidates {
                    let kind = MemoryKind::parse(&candidate.kind).unwrap_or(MemoryKind::Preference);
                    let text = one_line(candidate.text);
                    if text.is_empty() {
                        continue;
                    }
                    raw.push(Pending {
                        kind,
                        tags: normalize_tags(candidate.tags),
                        text,
                        source: source.clone(),
                    });
                }
            }
            Err(error) => {
                skipped += 1;
                eprintln!(
                    "\n  skip {}: {error:#}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
            }
        }
    }
    if progress {
        eprintln!();
    }
    if all_failed(sessions.len(), skipped) {
        bail!(
            "all {} session(s) failed; the inbox was left untouched",
            sessions.len()
        );
    }

    let existing = store.memories()?;
    let inbox = store.read_inbox()?;
    let merged = merge_candidates(&existing, inbox.as_deref(), raw);
    let added = merged.len().saturating_sub(
        inbox
            .as_deref()
            .map(parse_inbox)
            .map(|items| items.len())
            .unwrap_or(0),
    );

    write_inbox(store, &merged)?;
    println!(
        "{} new candidate(s), {} pending in {}",
        added,
        merged.len(),
        store.inbox_path().display()
    );
    if !merged.is_empty() {
        println!("运行 `niuse review` 逐条确认。");
    }
    Ok(())
}

/// Merge new candidates into the existing queue instead of replacing it.
///
/// Accepted memories and secret-bearing entries are dropped from both sides;
/// duplicates are compared with the same similarity rule as memory writes.
fn all_failed(selected: usize, skipped: usize) -> bool {
    selected > 0 && skipped == selected
}

fn merge_candidates(
    memories: &[MemoryEntry],
    inbox: Option<&str>,
    incoming: Vec<Pending>,
) -> Vec<Pending> {
    let mut merged: Vec<Pending> = inbox
        .map(parse_inbox)
        .unwrap_or_default()
        .into_iter()
        .filter(|pending| secret_reason(&pending.text).is_none())
        .filter(|pending| {
            !memories
                .iter()
                .any(|entry| conflicts(&entry.text, &entry.tags, &pending.text, &pending.tags))
        })
        .collect();

    for pending in incoming {
        if secret_reason(&pending.text).is_some()
            || memories
                .iter()
                .any(|entry| conflicts(&entry.text, &entry.tags, &pending.text, &pending.tags))
            || merged
                .iter()
                .any(|other| conflicts(&other.text, &other.tags, &pending.text, &pending.tags))
        {
            continue;
        }
        merged.push(pending);
    }
    merged
}

pub fn review(store: &Store) -> Result<()> {
    let Some(text) = store.read_inbox()? else {
        println!("没有待审队列。先运行 `niuse bootstrap`。");
        return Ok(());
    };
    let items = parse_inbox(&text);
    if items.is_empty() {
        println!("待审队列是空的。");
        return Ok(());
    }

    println!("{} 条待审。", items.len());
    let mut remaining: Vec<Pending> = Vec::new();
    let mut accepted = 0usize;
    let mut index = 0usize;

    while index < items.len() {
        let item = &items[index];
        println!();
        println!(
            "{} [{}] {}",
            item.kind.as_str(),
            if item.tags.is_empty() {
                "general".to_string()
            } else {
                item.tags.join(", ")
            },
            item.text
        );
        println!("  来源 {}", item.source);
        print!("[y] 接受  [n] 丢弃  [s] 留到下次  [q] 退出 > ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            remaining.extend(items[index..].iter().cloned());
            break;
        }
        match line.trim().to_lowercase().as_str() {
            "y" | "yes" => {
                if let Some(reason) = secret_reason(&item.text) {
                    println!("  拒绝：疑似包含 {reason}");
                } else {
                    let entry = MemoryEntry::new(
                        item.kind,
                        item.text.clone(),
                        item.tags.clone(),
                        Date::today(),
                        Some(item.source.clone()),
                    );
                    store.add_memory(entry)?;
                    record_metric(store, "review_accept", &item.source);
                    accepted += 1;
                    println!("  已接受");
                }
            }
            "s" | "skip" => remaining.push(item.clone()),
            "q" | "quit" => {
                remaining.extend(items[index..].iter().cloned());
                break;
            }
            _ => {
                record_metric(store, "review_discard", &item.source);
                println!("  已丢弃");
            }
        }
        index += 1;
    }

    write_inbox(store, &remaining)?;
    println!("\n接受 {} 条，剩余 {} 条。", accepted, remaining.len());
    Ok(())
}

fn record_metric(store: &Store, kind: &str, detail: &str) {
    if let Err(error) = store.append_metric(kind, detail) {
        eprintln!("warning: cannot record metric: {error:#}");
    }
}

fn write_inbox(store: &Store, items: &[Pending]) -> Result<()> {
    let mut text = String::from(
        "# 待审记忆\n\n<!-- 由 niuse bootstrap 生成；`niuse review` 处理后重写本文件。 -->\n\n",
    );
    if items.is_empty() {
        text.push_str("（没有待审的候选。）\n");
    }
    for item in items {
        let tags = if item.tags.is_empty() {
            String::new()
        } else {
            format!(" `{}`", item.tags.join(","))
        };
        text.push_str(&format!(
            "- [ ] `{}`{} {}  <!-- source: {} -->\n",
            item.kind.as_str(),
            tags,
            item.text,
            item.source
        ));
    }
    store.write_inbox(&text)
}

fn parse_inbox(text: &str) -> Vec<Pending> {
    text.lines().filter_map(parse_inbox_line).collect()
}

fn parse_inbox_line(line: &str) -> Option<Pending> {
    let rest = line.trim_start().strip_prefix("- [ ] ")?;
    let rest = rest.strip_prefix('`')?;
    let (kind, rest) = rest.split_once('`')?;
    let kind = MemoryKind::parse(kind)?;
    let rest = rest.trim_start();

    let (tags, rest) = if let Some(after) = rest.strip_prefix('`') {
        let (tags, after) = after.split_once('`')?;
        (
            tags.split(',')
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(str::to_string)
                .collect(),
            after.trim_start(),
        )
    } else {
        (Vec::new(), rest)
    };

    let (text, source) = match rest.rfind("<!-- source: ") {
        Some(index) => {
            let source = rest[index + "<!-- source: ".len()..]
                .trim()
                .trim_end_matches("-->")
                .trim()
                .to_string();
            (rest[..index].trim().to_string(), source)
        }
        None => (rest.trim().to_string(), String::new()),
    };

    (!text.is_empty()).then_some(Pending {
        kind,
        tags,
        text,
        source,
    })
}

async fn extract(provider: &dyn Provider, model: &str, transcript: &str) -> Result<Vec<Candidate>> {
    let text = complete(provider, model, EXTRACTION_SYSTEM, transcript, 1_200).await?;
    Ok(parse_candidates(&text))
}

async fn write_profile(
    store: &Store,
    provider: &dyn Provider,
    model: &str,
    sessions: &[(PathBuf, String)],
) -> Result<()> {
    let input = profile_input(sessions, MAX_PROFILE_CHARS);
    let text = complete(provider, model, PROFILE_SYSTEM, &input, 800).await?;
    let text = text.trim();
    if text.is_empty() {
        println!("profile: 模型没有返回内容");
        return Ok(());
    }
    let body = if text.starts_with("# ") {
        format!("{text}\n")
    } else {
        format!("# 关于我\n\n{text}\n")
    };
    if let Some(reason) = secret_reason(&body) {
        println!("profile: 拒绝写入，疑似包含 {reason}。");
        return Ok(());
    }
    store.write_user(&body)?;
    if let Err(error) = store.append_metric("profile", "") {
        eprintln!("warning: cannot record metric: {error:#}");
    }
    println!(
        "wrote {} — review with `niuse user edit`",
        store.user_path().display()
    );
    Ok(())
}

/// Concatenate session transcripts for one profile request, with a hard cap.
fn profile_input(sessions: &[(PathBuf, String)], max_chars: usize) -> String {
    let mut out = String::new();
    let mut used = 0usize;
    for (path, transcript) in sessions {
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("session");
        let block = format!("### {name}\n\n{transcript}\n\n");
        if used + block.chars().count() > max_chars {
            break;
        }
        used += block.chars().count();
        out.push_str(&block);
    }
    out
}

async fn complete(
    provider: &dyn Provider,
    model: &str,
    system: &str,
    input: &str,
    max_tokens: u32,
) -> Result<String> {
    let mut request = CompletionRequest::new(model);
    request.temperature = Some(0.0);
    request.max_tokens = Some(max_tokens);
    request.messages.push(ChatMessage::system(system));
    request.messages.push(ChatMessage::user(input));

    let mut stream = provider.stream(request).await?;
    let mut text = String::new();
    while let Some(event) = stream.next().await {
        match event.context("provider stream failed")? {
            StreamEvent::TextDelta(delta) => text.push_str(&delta),
            StreamEvent::Done { .. } => break,
            _ => {}
        }
    }
    Ok(text)
}

fn parse_candidates(raw: &str) -> Vec<Candidate> {
    let Some(start) = raw.find('[') else {
        return Vec::new();
    };
    let Some(end) = raw.rfind(']') else {
        return Vec::new();
    };
    serde_json::from_str(&raw[start..=end]).unwrap_or_default()
}

fn transcript(path: &Path) -> Result<Option<String>> {
    let loaded = load(path)?;
    let mut out = String::new();
    let mut budget = MAX_TRANSCRIPT_CHARS;

    for event in &loaded.events {
        let (role, text) = match event {
            Event::UserMessage { text, .. } => ("user", text),
            Event::AssistantMessage { text, .. } if !text.trim().is_empty() => ("assistant", text),
            _ => continue,
        };
        let text = truncate(text, MAX_MESSAGE_CHARS);
        let line = format!("[{role}] {text}\n\n");
        if line.len() > budget {
            break;
        }
        budget -= line.len();
        out.push_str(&line);
    }

    Ok((!out.trim().is_empty()).then_some(out))
}

fn recent_sessions(limit: usize, since_days: i64) -> Result<Vec<PathBuf>> {
    let directory = sessions_dir()?;
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(
            (since_days.max(1) as u64).saturating_mul(86_400),
        ))
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let mut files: Vec<(SystemTime, PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(&directory)
        .with_context(|| format!("read sessions {}", directory.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("jsonl") {
            continue;
        }
        let modified = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        if modified >= cutoff {
            files.push((modified, path));
        }
    }
    files.sort_by_key(|(modified, _)| Reverse(*modified));
    Ok(files
        .into_iter()
        .take(limit)
        .map(|(_, path)| path)
        .collect())
}

fn sessions_dir() -> Result<PathBuf> {
    if let Ok(home) = env::var("NIUCODE_HOME") {
        if !home.trim().is_empty() {
            return Ok(PathBuf::from(home).join("sessions"));
        }
    }
    let home = env::var_os("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home).join(".niucode").join("sessions"))
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max_chars).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inbox_round_trips() {
        let items = vec![
            Pending {
                kind: MemoryKind::Lesson,
                tags: vec!["rust".into(), "testing".into()],
                text: "用 cargo nextest".into(),
                source: "1792-1-0".into(),
            },
            Pending {
                kind: MemoryKind::Preference,
                tags: Vec::new(),
                text: "先给结论".into(),
                source: "1792-2-0".into(),
            },
        ];
        let mut text = String::from("# 待审记忆\n\n");
        for item in &items {
            let tags = if item.tags.is_empty() {
                String::new()
            } else {
                format!(" `{}`", item.tags.join(","))
            };
            text.push_str(&format!(
                "- [ ] `{}`{} {}  <!-- source: {} -->\n",
                item.kind.as_str(),
                tags,
                item.text,
                item.source
            ));
        }
        let parsed = parse_inbox(&text);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].kind, MemoryKind::Lesson);
        assert_eq!(parsed[0].tags, vec!["rust", "testing"]);
        assert_eq!(parsed[0].text, "用 cargo nextest");
        assert_eq!(parsed[0].source, "1792-1-0");
        assert_eq!(parsed[1].tags, Vec::<String>::new());
        assert_eq!(parsed[1].text, "先给结论");
    }

    #[test]
    fn profile_input_respects_the_cap() {
        let sessions = vec![
            (PathBuf::from("a.jsonl"), "a".repeat(600)),
            (PathBuf::from("b.jsonl"), "b".repeat(600)),
        ];
        let input = profile_input(&sessions, 700);
        assert!(input.contains("### a"));
        assert!(!input.contains("### b"));
        assert!(input.chars().count() <= 700);
    }

    #[test]
    fn all_failed_only_when_every_session_was_skipped() {
        assert!(all_failed(3, 3));
        assert!(!all_failed(3, 2));
        assert!(!all_failed(3, 0));
        assert!(!all_failed(0, 0));
    }

    #[test]
    fn merge_keeps_pending_and_drops_accepted() {
        let accepted = MemoryEntry::new(
            MemoryKind::Lesson,
            "already accepted",
            Vec::new(),
            Date::today(),
            None,
        );
        let inbox = "- [ ] `lesson` already accepted  <!-- source: s1 -->\n- [ ] `preference` keep me  <!-- source: s2 -->\n";
        let merged = merge_candidates(
            &[accepted],
            Some(inbox),
            vec![
                Pending {
                    kind: MemoryKind::Preference,
                    tags: Vec::new(),
                    text: "keep me".into(),
                    source: "s3".into(),
                },
                Pending {
                    kind: MemoryKind::Lesson,
                    tags: Vec::new(),
                    text: "new thought".into(),
                    source: "s4".into(),
                },
            ],
        );
        let texts: Vec<&str> = merged.iter().map(|pending| pending.text.as_str()).collect();
        assert_eq!(texts, vec!["keep me", "new thought"]);
    }

    #[test]
    fn candidate_json_is_extracted_from_chatter() {
        let raw =
            "好的：\n[{\"kind\":\"preference\",\"text\":\"用 nextest\",\"tags\":[\"rust\"]}]\n";
        let parsed = parse_candidates(raw);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].text, "用 nextest");
    }
}
