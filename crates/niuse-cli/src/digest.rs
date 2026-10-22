//! Turn a finished thread into a few journal bullets.
//!
//! The digest is deliberately narrow: user and assistant prose only, one
//! tool-free model call, at most five bullets, no write on a parse or secret
//! failure. The in-band journal remains the default; this is the opt-in path
//! for a long thread.

use std::path::Path;

use anyhow::{bail, Result};
use niucode_core::{load, Event};
use niucode_provider::Provider;
use niuse_core::model::one_line;
use niuse_core::policy::secret_reason;
use niuse_core::{Date, Store};

use crate::bootstrap::{complete, truncate};

const MAX_MESSAGE_CHARS: usize = 2_000;
const MAX_TRANSCRIPT_CHARS: usize = 12_000;
const MAX_BULLETS: usize = 5;

const DIGEST_SYSTEM: &str =
    "你是会话日记压缩器。根据下面这段 coding agent 与用户的会话，写 1-5 条 journal 条目。\n\
先判断：这段会话是否出现了一个决定、转折点、新出现的未解决问题、被说服、\n\
或判断/情绪的变化。如果只是完成例行任务、没有改变任何方向，只输出 NONE。\n\
不要写任务完成情况（“创建了/修复了/确认了 X”），也不要写工具调用过程；\n\
活动不是 journal。\n\
每条都要脱离这段会话也能看懂，一句话，具体；不要复制代码或文件内容，\n\
不要重复偏好，不要写密钥或别人的信息。\n\
没有值得记录的，就只输出 NONE。\n\
如果验证先失败后通过，记录失败的原因和修复，而不是只写最终通过。\n\
只输出以 \"- \" 开头的条目，不要解释，不要代码围栏。";

const DIGEST_GATE: &str =
    "判断下面这段 coding agent 与用户的会话，是否包含一个**新出现**的决定、\n\
转折点、新出现的未解决问题、被说服、或判断/情绪的变化。\n\
只算本会话中新出现的、或在本会话中被推进/改变的内容。\n\
如果只是任务完成、工具调用、确认结果，或只是重复/提醒会话开始前就存在的旧事项\n\
（例如一个早就记下的到期 loop），判断为 NO。\n\
如果只是复述/披露已经存在的记忆、偏好或事实，不是新决定或新问题，也判断为 NO。\n\
复述记忆不算新信息：记忆在会话开始前就存在；只有它被修改、被质疑、\n\
或被用于做出新决定，才算变化。仅复述，回答 NO。\n\
如果出现一次验证失败，随后被修复并通过，这也是本会话中的转折点，判断为 YES。\n\
只输出一行：`YES: 一句话理由` 或 `NO: 一句话理由`。不要输出其他内容。";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Gate {
    Yes,
    No,
}

fn parse_gate(raw: &str) -> Option<(Gate, String)> {
    let line = raw.trim().lines().next()?.trim();
    let (verdict, reason) = match line.split_once(':') {
        Some((verdict, reason)) => (verdict.trim(), reason.trim()),
        None => (line, ""),
    };
    let gate = match verdict.to_ascii_uppercase().as_str() {
        "YES" => Gate::Yes,
        "NO" => Gate::No,
        _ => return None,
    };
    Some((
        gate,
        if reason.is_empty() {
            "模型未给出理由".to_string()
        } else {
            reason.to_string()
        },
    ))
}

#[derive(Debug, Clone, Copy)]
pub struct DigestOptions {
    pub dry_run: bool,
    pub limit: usize,
}

pub async fn run(
    store: &Store,
    provider: &dyn Provider,
    model: &str,
    thread: &Path,
    project: &str,
    options: DigestOptions,
) -> Result<()> {
    let Some(transcript) = thread_transcript(thread, options.limit)? else {
        bail!("会话没有可压缩的用户或助手文本：{}", thread.display());
    };
    let input = format!("# 项目：{project}\n\n{transcript}");
    let gate = complete(provider, model, DIGEST_GATE, &input, 80).await?;
    let (verdict, reason) = parse_gate(&gate).ok_or_else(|| {
        anyhow::anyhow!("digest 预检没有返回 YES/NO（{}）；没有写入", gate.trim())
    })?;
    match verdict {
        Gate::No => {
            if options.dry_run {
                println!("digest dry run：没有可提炼的变化（{reason}），不写入");
            } else {
                println!("digest：没有可提炼的变化（{reason}），journal 未变");
            }
            return Ok(());
        }
        Gate::Yes => {
            println!("digest gate：YES · {reason}");
        }
    }
    let raw = complete(provider, model, DIGEST_SYSTEM, &input, 800).await?;
    let bullets = parse_bullets(&raw);
    if bullets.is_empty() {
        if raw.trim().eq_ignore_ascii_case("NONE") {
            if options.dry_run {
                println!("digest dry run：没有可提炼的变化，不写入");
            } else {
                println!("digest：没有可提炼的变化，journal 未变");
            }
            return Ok(());
        }
        bail!("模型没有返回 journal 条目；没有写入");
    }
    for bullet in &bullets {
        if let Some(reason) = secret_reason(bullet) {
            bail!("拒绝写入 digest：条目疑似包含 {reason}");
        }
    }

    let source = thread
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("thread");
    if options.dry_run {
        println!(
            "digest dry run：{} 条（来源 {source}），不写入",
            bullets.len()
        );
    } else {
        store.append_journal(Date::today(), &bullets)?;
        store.append_metric("digest", &format!("{source}:{}", bullets.len()))?;
        println!("journal +{} 条（来源 {source}）", bullets.len());
    }
    for bullet in &bullets {
        println!("  - {bullet}");
    }
    Ok(())
}

/// Use only the most recent messages, so a long thread does not digest its
/// opening instead of the work that just finished.
fn thread_transcript(path: &Path, limit: usize) -> Result<Option<String>> {
    let loaded = load(path)?;
    let messages: Vec<(&str, String)> = loaded
        .events
        .iter()
        .filter_map(|event| match event {
            Event::UserMessage { text, .. } if !text.trim().is_empty() => {
                Some(("user", text.clone()))
            }
            Event::AssistantMessage { text, .. } if !text.trim().is_empty() => {
                Some(("assistant", text.clone()))
            }
            Event::Verification {
                command, ok, step, ..
            } => Some((
                "verification",
                format!(
                    "{} {} {}",
                    step.as_deref().unwrap_or("-"),
                    if *ok { "ok" } else { "FAILED" },
                    command
                ),
            )),
            _ => None,
        })
        .collect();
    let start = messages.len().saturating_sub(limit.max(1));
    let mut out = String::new();
    let mut budget = MAX_TRANSCRIPT_CHARS;
    for (role, text) in &messages[start..] {
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

fn parse_bullets(raw: &str) -> Vec<String> {
    let mut bullets: Vec<String> = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        let Some(text) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) else {
            continue;
        };
        let text = one_line(text.to_string());
        if text.is_empty() || text.eq_ignore_ascii_case("none") || bullets.contains(&text) {
            continue;
        }
        bullets.push(text);
        if bullets.len() == MAX_BULLETS {
            break;
        }
    }
    bullets
}

#[cfg(test)]
mod tests {
    use super::*;
    use niucode_core::{now_ms, SessionLog};
    use niucode_provider::{ScriptedProvider, StreamEvent};
    use std::path::PathBuf;

    fn turn(text: &str) -> Vec<StreamEvent> {
        vec![
            StreamEvent::TextDelta(text.into()),
            StreamEvent::Done {
                finish_reason: Some("stop".into()),
            },
        ]
    }

    fn scripted(text: &str) -> ScriptedProvider {
        ScriptedProvider::from_turns(vec![turn("YES: decision"), turn(text)])
    }

    fn thread_with(messages: &[&str]) -> (tempfile::TempDir, PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let log = SessionLog::create(directory.path()).unwrap();
        for message in messages {
            log.append(&Event::UserMessage {
                text: (*message).into(),
                ts: now_ms(),
            })
            .unwrap();
            log.append(&Event::AssistantMessage {
                text: "ok".into(),
                reasoning: None,
                tool_calls: Vec::new(),
                ts: now_ms(),
            })
            .unwrap();
        }
        let path = log.path().to_path_buf();
        drop(log);
        (directory, path)
    }

    fn store(directory: &Path) -> Store {
        let store = Store::at(directory.join(".niuse"));
        store.init().unwrap();
        store
    }

    #[test]
    fn parses_gate_verdict_and_reason() {
        assert_eq!(
            parse_gate("YES: a decision"),
            Some((Gate::Yes, "a decision".into()))
        );
        assert_eq!(
            parse_gate("no: old reminder"),
            Some((Gate::No, "old reminder".into()))
        );
        assert_eq!(
            parse_gate("YES"),
            Some((Gate::Yes, "模型未给出理由".into()))
        );
        assert_eq!(parse_gate("MAYBE: x"), None);
        assert_eq!(parse_gate(""), None);
    }

    #[test]
    fn parses_bullets_dedupes_and_caps() {
        let raw =
            "Here:\n```\n- first\n* second\n- first\n- third\n- fourth\n- fifth\n- sixth\n```\n";
        assert_eq!(
            parse_bullets(raw),
            vec!["first", "second", "third", "fourth", "fifth"]
        );
        assert!(parse_bullets("NONE").is_empty());
        assert!(parse_bullets("just chatter").is_empty());
    }

    #[test]
    fn transcript_includes_verification_evidence() {
        let (directory, thread) = thread_with(&["run the check"]);
        let log = SessionLog::open(&thread).unwrap();
        log.append(&Event::Verification {
            command: "cargo test".into(),
            ok: false,
            output: "one failure".into(),
            step: Some("p-1".into()),
            ts: now_ms(),
        })
        .unwrap();
        let transcript = thread_transcript(&thread, 20).unwrap().unwrap();
        assert!(transcript.contains("[verification] p-1 FAILED cargo test"));
        drop(log);
        drop(directory);
    }

    #[test]
    fn transcript_prefers_recent_messages() {
        let (directory, thread) = thread_with(&[
            "message 0",
            "message 1",
            "message 2",
            "message 3",
            "message 4",
        ]);
        let transcript = thread_transcript(&thread, 4).unwrap().unwrap();
        assert!(!transcript.contains("message 0"));
        assert!(!transcript.contains("message 2"));
        assert!(transcript.contains("message 3"));
        assert!(transcript.contains("message 4"));
        drop(directory);
    }

    #[tokio::test]
    async fn writes_at_most_five_bullets() {
        let (directory, thread) = thread_with(&["finish the digest"]);
        let store = store(directory.path());
        let provider = scripted(
            "- decision one\n- decision two\n- decision three\n- decision four\n- decision five\n- decision six",
        );
        run(
            &store,
            &provider,
            "scripted",
            &thread,
            "niuse",
            DigestOptions {
                dry_run: false,
                limit: 20,
            },
        )
        .await
        .unwrap();

        let days = store.recent_journal(1).unwrap();
        assert_eq!(days.len(), 1);
        assert_eq!(days[0].bullets.len(), 5);
        assert_eq!(days[0].bullets[0], "decision one");
    }

    #[tokio::test]
    async fn dry_run_does_not_write() {
        let (directory, thread) = thread_with(&["finish the digest"]);
        let store = store(directory.path());
        let provider = scripted("- a decision");
        run(
            &store,
            &provider,
            "scripted",
            &thread,
            "niuse",
            DigestOptions {
                dry_run: true,
                limit: 20,
            },
        )
        .await
        .unwrap();
        assert!(store.recent_journal(10).unwrap().is_empty());
    }

    #[tokio::test]
    async fn refuses_a_secret_without_writing() {
        let (directory, thread) = thread_with(&["use the key"]);
        let store = store(directory.path());
        let provider = scripted("- leaked key sk-abcdefghijklmnopqrstuvwx");
        let error = run(
            &store,
            &provider,
            "scripted",
            &thread,
            "niuse",
            DigestOptions {
                dry_run: false,
                limit: 20,
            },
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("拒绝"));
        assert!(store.recent_journal(10).unwrap().is_empty());
    }

    #[tokio::test]
    async fn none_is_a_no_change_and_writes_nothing() {
        let (directory, thread) = thread_with(&["nothing durable"]);
        let store = store(directory.path());
        let provider = scripted("NONE");
        run(
            &store,
            &provider,
            "scripted",
            &thread,
            "niuse",
            DigestOptions {
                dry_run: false,
                limit: 20,
            },
        )
        .await
        .unwrap();
        assert!(store.recent_journal(10).unwrap().is_empty());
        assert!(!store.metrics().unwrap().contains("digest"));
    }

    #[tokio::test]
    async fn gate_no_skips_the_digest_call() {
        let (directory, thread) = thread_with(&["routine task"]);
        let store = store(directory.path());
        let provider = ScriptedProvider::from_turns(vec![turn("NO: no durable change")]);
        run(
            &store,
            &provider,
            "scripted",
            &thread,
            "niuse",
            DigestOptions {
                dry_run: false,
                limit: 20,
            },
        )
        .await
        .unwrap();
        assert!(store.recent_journal(10).unwrap().is_empty());
        assert!(!store.metrics().unwrap().contains("digest"));
    }

    #[tokio::test]
    async fn malformed_answer_is_still_an_error() {
        let (directory, thread) = thread_with(&["nothing durable"]);
        let store = store(directory.path());
        let provider = scripted("just some prose, no bullets");
        let error = run(
            &store,
            &provider,
            "scripted",
            &thread,
            "niuse",
            DigestOptions {
                dry_run: false,
                limit: 20,
            },
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("没有返回"));
        assert!(store.recent_journal(10).unwrap().is_empty());
    }
}
