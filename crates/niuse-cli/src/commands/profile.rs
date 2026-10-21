//! Identity and user-profile commands.

use std::io::{self, Write};

use anyhow::{bail, Result};
use niuse_core::policy::secret_reason;
use niuse_core::Store;

use super::edit_markdown;

pub fn user_show(store: &Store, json: bool) -> Result<()> {
    let text = store.user()?;
    if json {
        let payload = serde_json::json!({
            "path": store.user_path().display().to_string(),
            "text": text,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    println!("{}", store.user_path().display());
    match text {
        Some(text) => print!("{text}"),
        None => println!("(missing; run `niuse bootstrap --profile` or `niuse user edit`)"),
    }
    Ok(())
}

pub fn user_edit(store: &Store) -> Result<()> {
    let current = store.user()?.unwrap_or_else(|| {
        "# 关于我\n\n- 我在做什么：\n- 我的工作方式：\n- 我的雷区：\n".to_string()
    });
    let Some(updated) = edit_markdown(&current) else {
        println!("no change");
        return Ok(());
    };
    if let Some(reason) = secret_reason(&updated) {
        bail!("refusing to save user.md: looks like a {reason}");
    }
    store.write_user(&updated)?;
    if let Err(error) = store.append_metric("user_edit", "") {
        eprintln!("warning: cannot record metric: {error:#}");
    }
    println!("updated {}", store.user_path().display());
    Ok(())
}

pub fn identity_edit(store: &Store) -> Result<()> {
    let current = store
        .identity()?
        .unwrap_or_else(|| "# 你是 niuse\n\n描述它是谁、它的边界和说话方式。\n".to_string());
    let Some(updated) = edit_markdown(&current) else {
        println!("no change");
        return Ok(());
    };
    if let Some(reason) = secret_reason(&updated) {
        bail!("refusing to save identity.md: looks like a {reason}");
    }
    store.write_identity(&updated)?;
    if let Err(error) = store.append_metric("identity_edit", "") {
        eprintln!("warning: cannot record metric: {error:#}");
    }
    println!("updated {}", store.identity_path().display());
    Ok(())
}

pub fn identity_show(store: &Store, json: bool) -> Result<()> {
    let text = store.identity()?;
    if json {
        let payload = serde_json::json!({
            "path": store.identity_path().display().to_string(),
            "text": text,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    match text {
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
    if let Some(reason) = secret_reason(&text) {
        bail!("refusing to save identity.md: looks like a {reason}");
    }
    store.write_identity(&text)?;
    println!("wrote {}", path.display());
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
