//! Store-only commands. None of these construct a provider, so they work
//! offline and without credentials.

use anyhow::{bail, Context, Result};
use niuse_core::model::one_line;
use niuse_core::policy::secret_reason;
use niuse_core::prompt::compose;
use niuse_core::{Date, MemoryEntry, MemoryKind, Store};
use std::io::{self, Write};

pub fn mem_list(store: &Store, all: bool) -> Result<()> {
    let today = Date::today();
    let memories = store.memories()?;
    let shown: Vec<&MemoryEntry> = memories
        .iter()
        .filter(|entry| all || !entry.is_expired(today))
        .collect();

    let stale = memories
        .iter()
        .filter(|entry| entry.is_stale(today) && !entry.is_expired(today))
        .count();
    let expired = memories
        .iter()
        .filter(|entry| entry.is_expired(today))
        .count();
    println!(
        "{} 条记忆 · {} 条待验证 · {} 条已过期（不再自动注入）",
        memories.len(),
        stale,
        expired
    );
    if shown.is_empty() {
        println!("（空。用 `niuse mem add \"...\"` 或等待对话中的候选。）");
        return Ok(());
    }
    for entry in shown {
        let age = entry.age_days(today);
        let age_label = match age {
            0 => "今天".to_string(),
            1..=30 => format!("{age} 天前"),
            _ if entry.is_expired(today) => format!("{age} 天前 · 已过期"),
            _ => format!("{age} 天前 · 待验证"),
        };
        let tags = if entry.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", entry.tags.join(", "))
        };
        println!(
            "m-{}  {}{}  {}  {}",
            entry.id,
            entry.kind.as_str(),
            tags,
            age_label,
            entry.text
        );
    }
    Ok(())
}

pub fn mem_add(
    store: &Store,
    text: &str,
    kind: &str,
    tags: Vec<String>,
    force: bool,
) -> Result<()> {
    let kind = MemoryKind::parse(kind)
        .with_context(|| format!("unknown kind {kind:?}; use preference or lesson"))?;
    let text = one_line(text.trim().to_string());
    if text.is_empty() {
        bail!("memory text is empty");
    }
    if let Some(reason) = secret_reason(&text) {
        if !force {
            bail!("refusing to store: looks like a {reason}; pass --force to override");
        }
        eprintln!("warning: overriding secret scan ({reason})");
    }
    let entry = store.add_memory(MemoryEntry::new(kind, text, tags, Date::today(), None))?;
    println!("m-{}", entry.id);
    Ok(())
}

pub fn mem_forget(store: &Store, id: &str) -> Result<()> {
    let id = normalize_id(id);
    if store.remove_memory(&id)? {
        println!("forgot m-{id}");
    } else {
        bail!("no memory m-{id}");
    }
    Ok(())
}

pub fn mem_confirm(store: &Store, id: &str) -> Result<()> {
    let id = normalize_id(id);
    let updated = store.update_memory(&id, |entry| entry.confirmed = Date::today())?;
    match updated {
        Some(entry) => println!("m-{} confirmed {}", entry.id, entry.confirmed),
        None => bail!("no memory m-{id}"),
    }
    Ok(())
}

pub fn mem_review(store: &Store) -> Result<()> {
    let today = Date::today();
    let stale: Vec<MemoryEntry> = store
        .memories()?
        .into_iter()
        .filter(|entry| entry.is_stale(today))
        .collect();
    if stale.is_empty() {
        println!("没有需要复核的记忆。");
        return Ok(());
    }

    for entry in stale {
        println!();
        println!(
            "m-{} · {} · 最后确认 {}（{} 天前）",
            entry.id,
            entry.kind.as_str(),
            entry.confirmed,
            entry.age_days(today)
        );
        println!("  {}", entry.text);
        print!("[y] 仍然成立  [d] 删除  [s] 跳过  [q] 退出 > ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            break;
        }
        match line.trim().to_lowercase().as_str() {
            "y" | "yes" => {
                store.update_memory(&entry.id, |entry| entry.confirmed = today)?;
                println!("  已确认");
            }
            "d" | "delete" => {
                store.remove_memory(&entry.id)?;
                println!("  已删除");
            }
            "q" | "quit" => break,
            _ => println!("  跳过"),
        }
    }
    Ok(())
}

pub fn loops_list(store: &Store, all: bool) -> Result<()> {
    let today = Date::today();
    let loops = store.loops()?;
    let open = loops.iter().filter(|item| !item.done).count();
    println!("{} 条承诺 · {} 条未完成", loops.len(), open);
    for item in &loops {
        if item.done && !all {
            continue;
        }
        let mark = if item.done { "x" } else { " " };
        let due = match item.due {
            Some(due) if item.is_overdue(today) => format!("（截止 {due}，已逾期）"),
            Some(due) => format!("（截止 {due}）"),
            None => String::new(),
        };
        println!("[{mark}] l-{} {}{due}", item.id, item.text);
    }
    Ok(())
}

pub fn loops_done(store: &Store, id: &str) -> Result<()> {
    let id = normalize_id(id);
    if store.close_loop(&id)? {
        println!("closed l-{id}");
    } else {
        bail!("no loop l-{id}");
    }
    Ok(())
}

pub fn journal_show(store: &Store, days: usize) -> Result<()> {
    let days = store.recent_journal(days)?;
    if days.is_empty() {
        println!("（还没有 journal。）");
        return Ok(());
    }
    for day in days {
        println!("## {}", day.date);
        for bullet in &day.bullets {
            println!("- {bullet}");
        }
        println!();
    }
    Ok(())
}

pub fn identity_show(store: &Store) -> Result<()> {
    match store.identity()? {
        Some(text) => {
            println!("{}", store.identity_path().display());
            println!();
            print!("{text}");
        }
        None => println!(
            "还没有 identity.md。运行 `niuse identity init` 创建：{}",
            store.identity_path().display()
        ),
    }
    Ok(())
}

pub fn identity_init(store: &Store, force: bool) -> Result<()> {
    let path = store.identity_path();
    if path.exists() && !force {
        bail!(
            "{} already exists; pass --force to overwrite",
            path.display()
        );
    }

    let answers = [
        ask("你叫它什么名字", "niuse")?,
        ask("它是你的什么（搭档 / 同事 / 朋友）", "工程搭档")?,
        ask("用三个词描述它的性格", "直接、克制、有判断")?,
        ask("它应该坚持的一条原则", "不知道就说不知道")?,
        ask("它绝对不能做的一件事", "奉承我、替我做决定")?,
        ask("它说话的方式", "先给结论，再给理由；不说客套话")?,
        ask("它应该主动提醒你什么", "未完成的承诺和明显的风险")?,
        ask("它怎么称呼你", "你")?,
    ];

    let name = &answers[0];
    let relationship = &answers[1];
    let style = &answers[5];
    let text = format!(
        "# 你是 {name}\n\n\
         你是一个{relationship}，不是一个通用助手。你称呼对方为「{user}」。\n\n\
         ## 性格\n{persona}\n\n\
         ## 坚持\n- {principle}\n\n\
         ## 边界\n- {boundary}\n\n\
         ## 说话方式\n{style}\n\n\
         ## 主动提醒\n{reminder}\n",
        user = answers[7],
        persona = answers[2],
        principle = answers[3],
        boundary = answers[4],
        style = style,
        reminder = answers[6],
    );
    store.write_identity(&text)?;
    println!("wrote {}", path.display());
    Ok(())
}

pub fn status(store: &Store) -> Result<()> {
    let today = Date::today();
    let memories = store.memories()?;
    let loops = store.loops()?;
    let journal = store.recent_journal(3)?;
    let identity = store.identity()?;
    let user = store.user()?;
    let composition = compose(
        identity.as_deref(),
        user.as_deref(),
        &memories,
        &loops,
        &journal,
        today,
    );

    println!("home      {}", store.root().display());
    println!("thread    {}", store.session_path().display());
    println!(
        "identity  {}",
        if identity.is_some() {
            "已设置"
        } else {
            "未设置"
        }
    );
    println!(
        "memory    {} 条（{} 条待验证，{} 条已过期）",
        memories.len(),
        memories
            .iter()
            .filter(|entry| entry.is_stale(today) && !entry.is_expired(today))
            .count(),
        memories
            .iter()
            .filter(|entry| entry.is_expired(today))
            .count()
    );
    println!(
        "loops     {} 条未完成",
        loops.iter().filter(|item| !item.done).count()
    );
    println!(
        "prompt    ~{} tokens（注入 {} 条记忆，{} 条被预算或过期挡掉）",
        composition.tokens, composition.memory_injected, composition.memory_dropped
    );
    Ok(())
}

fn ask(prompt: &str, default: &str) -> Result<String> {
    print!("{prompt} [{default}]: ");
    let _ = io::stdout().flush();
    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        return Ok(default.to_string());
    }
    let answer = line.trim();
    Ok(if answer.is_empty() {
        default.to_string()
    } else {
        answer.to_string()
    })
}

fn normalize_id(id: &str) -> String {
    id.trim()
        .trim_start_matches("m-")
        .trim_start_matches('l')
        .trim_start_matches('-')
        .to_string()
}
