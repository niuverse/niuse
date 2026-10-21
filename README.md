# niuse

**niucode 是身体，niuse 是身份。**

niuse 是一个站在 [niucode](https://github.com/niuverse/niucode) 之上的个人工程伙伴。它把你的判断、偏好和踩过的坑沉淀成可读、可改、可回滚的 Markdown，让同一个它跨项目、跨会话地记得你、追踪自己答应过的事，并在你判断错的时候开口。

> 状态：**MVP 可运行**。五件套、候选协议、软过期、历史冷启动、REPL 都已实现；缺的是真实终端里的长期 dogfood。

## 快速开始

需要 Rust 1.85+，以及本机 `../niuverse/niucode` 的检出（当前用 path 依赖；后续换成钉死 rev 的 git 依赖）。

```sh
cargo build --release

./target/release/niuse                     # 进入 REPL
./target/release/niuse identity init       # 八问，写 ~/.niuse/identity.md
./target/release/niuse status              # 看看现在注入多少 token
./target/release/niuse bootstrap --limit 5 # 从旧 niucode session 里挖候选
./target/release/niuse review              # 逐条确认
```

模型凭证复用 niucode：

```sh
niucode auth set deepseek
```

## 它记得什么

```text
~/.niuse/
  identity.md      它是谁（你写，它只能提建议）
  user.md          你是谁（冷启动时从历史会话挖，你审）
  memory/          事实：偏好 / 决策 / 踩坑（要确认，会过期）
  journal/         经历：每次会话后它自己写（不确认）
  loops.md         承诺：它说要做的事（带截止）
  thread.jsonl     关系主线（可恢复的 niucode session）
  inbox.md         待审队列（bootstrap 的产物）
```

注入顺序固定为稳定 → 易变：`identity → user → memory → protocol → today → loops → journal`。整个 system prompt 每个 session 只组装一次，避免每轮击穿 prompt cache。

**写入门槛分级**：

| 写什么 | 谁决定 | 要不要确认 |
|---|---|---|
| journal、loops | 它自己 | 不要 |
| memory | 它提议 | 要 |
| identity | 你 | 只有你能改 |

## 命令

全局参数：`--preset`、`--model`、`--no-thinking`、`--cwd`、`--sandbox`、`--approval`、`--network`、`--home`、`--max-steps`。

```text
niuse                         交互式 REPL（恢复同一条主线）
niuse exec "<task>"           跑一个 turn，候选只打印不写入
niuse identity init|show
niuse mem list|add|forget|confirm|review
niuse loops list|done
niuse journal [--days N]
niuse bootstrap [--limit N] [--since-days N]
niuse review
niuse status
```

REPL 内：`/help`、`/mem`、`/mem review`、`/remember`、`/forget`、`/loops`、`/done`、`/journal`、`/identity`、`/cd`、`/status`、`/prompt`、`/quit`。

## 它是怎么工作的

一个 turn：

```text
读五件套 → 组装 prompt（一次）→ run_with_history
        → 流式渲染，尾部缓冲剥离协议块
        → journal / loop 自动落盘
        → memory 候选：y / n / e / s
```

模型在回答末尾可以带三种块，niuse 解析后从可见文本里剥离：

```text
<<niuse:memory kind=preference tags=rust,testing>>
一句话结论
<</niuse:memory>>

<<niuse:loop due=2026-10-22>>
答应过的事
<</niuse:loop>>

<<niuse:journal>>
- 今天与上次不同的决定或转折
<</niuse:journal>>
```

不额外调用模型。漏记是可接受的，错记不是。

## 代码结构

```text
crates/
  niuse-core/   纯领域：日期、五件套解析/写回、prompt、协议块、过期/冲突/密钥扫描
  niuse-cli/    REPL、命令、provider 装配、session 驱动、bootstrap
```

`niuse-core` 没有 IO 副作用（存储层除外），测试不需要 API key。当前共 41 个单测（core 35、bootstrap 2、看板 4）。`niuse-cli` 用一层约 200 行的 driver 把 niucode 的 API 挡在外面。

## 架构看板

```sh
cargo xtask dashboard          # 生成 docs/index.html
cargo xtask dashboard --check  # 生成物过期或引用断开时失败
```

四个视图：Architecture（默认）、Work、Docs、Decisions。源文件是 `docs/dashboard/project.json`、`docs/board/*.md` 和 `docs/**/*.md`；维护方式见 [docs/README.md](docs/README.md)。

## 明确不做

常驻进程、Web UI、微信 / 飞书通道、定时推送、向量检索、session 全量索引、自动改 repo、多人协作、TUI（MVP 阶段）、通用 agent 框架。

## 已知限制

- `NIUSE_MEMORY=off` 关闭记忆注入与候选，用于 A/B 验证。

- 单线程：一条 `thread.jsonl`，没有 `/new`。
- 审批只打印摘要和增删行数，还没有结构化 diff 预览。
- 记忆上限约 200 条，超出靠预算裁剪和人工复核。
- 日期按 UTC 计算，午夜附近可能差一天。
- `bootstrap` 会把旧会话的 user/assistant 文本发给 provider（不含工具输出）。
- 相似度用字符 bigram Jaccard，阈值是启发式，跑两周后再调。

## 文档

- [实施计划](docs/plan.md)：决策、五件套、turn 生命周期、里程碑与验收。
- [ADR-0001：记忆层](docs/adr/0001-memory-layer.md)
- [ADR-0002：身份层](docs/adr/0002-identity-layer.md)
