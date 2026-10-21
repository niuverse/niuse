+++
id = "architecture"
title = "架构"
kind = "doc"
status = "living"
order = 5
summary = "niuse 的边界、五件套、turn 数据流、写入分级与明确不做的事。"
+++

## 一句话

niuse 是 niucode 之上的身份与记忆层：它决定「你是谁、它记得什么、它答应过什么」，但把「能不能做」留给 niucode 的 loop、工具与沙箱。

## 依赖方向

```text
crates/niuse-cli     REPL / 命令 / renderer / driver / bootstrap
    │
crates/niuse-core    date / model / store / prompt / candidate / policy
    │
niucode-agent        run_with_history / AgentConfig / project_history
niucode-core         SessionLog / Event
niucode-provider     Provider / ChatMessage
niucode-auth         AuthStore
niucode-tools        SandboxPolicy / ApprovalMode
```

只有 `driver.rs` 直接接触 `niucode-agent`。其余代码要么是纯领域逻辑，要么只认 `niuse-core` 的类型。这样 niucode 升级时，需要改的地方被压在一个文件里。

不 fork niucode，也不给它的 loop 加 memory 钩子。记忆是产品层，内核保持最小。

## 模块

| 模块 | 层 | 职责 |
|---|---|---|
| `cli` | Interface | 参数、子命令、离线命令、装配 |
| `repl` | Interface | 对话、斜杠命令、候选确认、渲染 |
| `prompt` | Engine | 稳定前缀的 system prompt 与预算 |
| `protocol` | Engine | 带内块的解析与流式剥离 |
| `policy` | Engine | 软过期、冲突、密钥扫描 |
| `driver` | Adapters | niucode 边界、session、per-turn cwd |
| `bootstrap` | Adapters | 旧 session 抽取与 inbox |
| `stores` | State | 五件套的读写与权限 |
| `dashboard` | Engineering | 离线看板生成 |

## 一个 turn 的数据流

```text
启动
  ├─ 读 identity / user / memory / loops / journal
  ├─ compose() 组装 system prompt（每个 session 一次）
  └─ project_history(thread.jsonl) 恢复历史

用户输入
  ├─ run_with_history(config, prompt, history)
  ├─ AgentEvent → Renderer（TextDelta 经过 StreamFilter）
  ├─ 尾部缓冲剥离 <<niuse:*>> 块
  ├─ journal / loop：自动写
  ├─ memory：打印候选 → y / n / e / s
  └─ 重新投影 history，进入下一轮
```

`history` 不另存一份。它由 niucode 的 `project_history` 从事件日志投影，所以 compaction、replay、fork 的行为与 niucode 完全一致。

## 五件套与注入

```text
~/.niuse/
  identity.md      它是谁（人写，它只能提建议）
  user.md          你是谁（冷启动挖，人审）
  memory/          事实：偏好 / 决策 / 踩坑（确认，会过期）
  journal/         经历：按月分文件，按天合并（自动）
  loops.md         承诺（自动，带截止）
  thread.jsonl     关系主线
  inbox.md         bootstrap 的待审队列
```

注入顺序固定为稳定 → 易变：

| 段 | 预算 | 变化频率 |
|---|---|---|
| persona | 常量 | 无 |
| identity | ≤ 600 tok | 极低 |
| user | ≤ 600 tok | 低 |
| memory | ≤ 2 000 tok | 低 |
| protocol | 常量 | 无 |
| today | 一行 | 每天 |
| loops | ≤ 300 tok | 中 |
| journal | ≤ 600 tok | 高 |

前四段加上 protocol 构成稳定的 cache 前缀；`today` 之后才是每 session 可能变化的内容。system prompt 只组装一次，session 中途不重建。

## 写入分级与协议

| 写什么 | 谁决定 | 确认 |
|---|---|---|
| journal、loops | 它自己 | 不要 |
| memory | 它提议 | 要 |
| identity | 人 | 只有人能改 |

模型在回答末尾可以带三种块：

```text
<<niuse:memory kind=preference tags=rust,testing>>
一句话结论
<</niuse:memory>>

<<niuse:loop due=2026-10-22>>
答应过的事
<</niuse:loop>>

<<niuse:journal>>
- 与上次不同的决定或转折
<</niuse:journal>>
```

`memory` 每轮最多一条；被拒绝后同标签在本次 session 不再提。`journal` 只记变化，不记流水账。

## 安全与隐私

- `~/.niuse` 目录 `0700`，文件 `0600`，写临时文件再 rename。
- 密钥模式扫描在写入前执行；命中即拒，CLI 需要 `--force` 才能覆盖。
- 记忆每一轮都会发给 provider，所以泄露面比普通文件大。
- 项目知识留在 repo 的 `AGENTS.md`，niuse 不自动改仓库。
- `NIUSE_MEMORY=off` 关闭记忆注入与候选，用于 A/B 验证和信任受损时的降级。

## 明确不做

常驻进程、Web UI、微信 / 飞书通道、定时推送、向量检索、session 全量索引、自动改 repo、多人协作、TUI（MVP 阶段）、通用 agent 框架。

判断标准只有一句：**它是在塑造身份，还是只是在堆能力？**
