+++
id = "plan"
title = "niuse 实施计划"
kind = "plan"
status = "active"
order = 10
summary = "十五个决策、五件套、turn 生命周期、写入分级、三周计划与可验证的验收标准。"
+++

## 一句话

在 niucode 之上加一层持久的身份与记忆，让同一个 agent 跨项目、跨会话地认识你；不做常驻服务，不做向量检索，不自动改你的仓库。

## 1. 定位

niucode 负责「能做」：loop、工具、沙箱、事件日志。niuse 负责「是谁」：它记得什么、答应过什么、什么时候该反对你。

判断一个功能该不该进 niuse，只问一句：**它是在塑造身份，还是只是在堆能力？** 堆能力的，留给 niucode 或干脆不做。

## 2. 决策表

| # | 问题 | 决策 |
|---|---|---|
| D1 | 定位 | 跨项目工程记忆 |
| D2 | 记忆内容 | 偏好 + 决策 + 踩坑；少量、高价值、可审阅 |
| D3 | 归属 | 全局偏好进 niuse；项目知识留在 repo 的 `AGENTS.md` |
| D4 | 写入 | agent 提议 + 一键确认，每 turn ≤ 1 条 |
| D5 | 读取 | 全量注入 + 作用域过滤 |
| D6 | 架构 | 独立 repo，依赖 niucode lib，自己驱动 turn |
| D7 | 存储 | `~/.niuse/` 纯 Markdown，0600 |
| D8 | 界面 | 交互式 REPL；TUI 后置 |
| D9 | 过期 | 软过期 + 冲突时替换 |
| D10 | 隐私 | 密钥模式黑名单，命中即拒 |
| D11 | MVP | 只做全局偏好一条链 |
| D12 | 头号风险 | 确认疲劳 |
| D13 | 形态 | 身份层 + 可恢复会话，无常驻进程 |
| D14 | 冷启动 | 从历史 session 挖掘；`identity` 由你定 |
| D15 | 性格 | 有观点的合作者 |

每条决策的取舍理由见 [ADR-0001](adr/0001-memory-layer.md) 与 [ADR-0002](adr/0002-identity-layer.md)。

### 记忆内容的准入标准

一条信息要进 `memory/`，必须同时满足：

1. 跨会话仍然有用；
2. 在 repo 里 grep 不出来；
3. 一句话说得清，人能审阅。

任何一条不满足就不进。这条标准的作用是把「全量 session 索引」和「环境事实」挡在门外——它们看起来像记忆，实际上是检索和配置。

## 3. 文件与注入

### 3.1 五件套

```text
~/.niuse/
  identity.md          它是谁：名字、口吻、价值观、边界
  user.md              你是谁：目标、节奏、品味、雷区、在做的事
  memory/
    preferences.md     偏好
    lessons.md         决策与踩坑
  journal/
    2026-10.md         经历，按月一个文件
  loops.md             承诺：它说要做的事
```

- 全部为 UTF-8 Markdown，目录 `0700`，文件 `0600`。
- `memory/` 可以单独 `git init`，用于备份和看 diff；`journal/` 默认不提交。
- 不做数据库。单用户、200 条以内，文件就是最好的查询引擎。

### 3.2 注入顺序与预算

顺序固定为**稳定 → 易变**，且 system prompt 每个 session 只组装一次（session 中途不重建，避免每 turn 触发 cache miss）：

| 段 | 来源 | 预算 | 变化频率 |
|---|---|---|---|
| 身份 | `identity.md` | ≤ 600 tok | 极低 |
| 用户 | `user.md` | ≤ 600 tok | 低 |
| 事实 | `memory/*.md` | ≤ 2 000 tok | 低 |
| 承诺 | `loops.md` | ≤ 300 tok | 中 |
| 最近经历 | `journal/` 最近 3 条 | ≤ 600 tok | 高 |

总预算约 4k token。超预算时先丢最老的 `unverified` 事实，再截断 journal，绝不静默丢 `identity` 或 `user`。

### 3.3 条目格式

`memory/` 里一条记忆是一个小节，机器可解析、人可读：

```markdown
### m-7f3a

- 结论：Rust 仓库一律用 `cargo nextest`，不用 `cargo test`
- 标签：rust, testing
- 创建：2026-10-21
- 最后确认：2026-10-21
- 来源：`1792493790852-350850-0`
```

`id` 取随机四位十六进制，稳定不复用。`来源` 指向 niucode 的 session id，是唯一的审计线索。

`loops.md`：

```markdown
- [ ] 确认 niucode 是否支持 per-turn cwd（2026-10-22）
- [x] 给 bootstrap 加采样上限（2026-10-21）
```

## 4. 一个 turn 的生命周期

```text
启动
  ├─ 读五件套
  ├─ 组装 system prompt（一次性）
  └─ 恢复主线程 history（project_history）

每轮输入
  ├─ run_with_history(config, prompt, history)
  ├─ 流式渲染事件（自有 renderer）
  ├─ 尾部缓冲：剥离 <<niuse:memory>> / <<niuse:loop>> 块
  ├─ 有候选 → 打印 → y / n / e
  ├─ 接受 → 写 memory（或 loops）
  └─ 追加到 journal 缓冲

退出 / 切换项目
  └─ journal 落盘（3–5 行）
```

`history` 由 niucode 的 `project_history` 从 session 事件投影，不另存一份对话。这样 replay、fork、compaction 的行为与 niucode 完全一致，niuse 不重新发明。

## 5. 写入分级

### 5.1 候选块协议

不额外调用模型。system prompt 里约定：如果本轮产生了值得长期记住的东西，回答末尾可以带一个块：

```text
<<niuse:memory scope=global tags=rust,testing>>
Rust 仓库一律用 cargo nextest，不用 cargo test
<</niuse:memory>>
```

规则：

- 每轮最多一个块；没有就不提，**允许连续多轮无候选**。
- 允许 `memory`、`loop`、`journal` 三种块。`journal` 与 `loop` 自动落盘，不打断你；只有 `memory` 需要确认。
- `memory` 块被拒绝后，同标签的候选在本次 session 内不再提。
- 解析失败按「无候选」处理，不报错、不猜测。

代价是模型可能漏记。这与 D4、D9 是同一条价值观：**宁可漏记，不可错记**。若 dogfood 两周后漏记严重，再引入离线 proposer，而不是现在就把成本加上去。

### 5.2 确认交互

```text
记忆候选 [global · rust,testing]
  Rust 仓库一律用 cargo nextest，不用 cargo test
  [y] 接受   [n] 丢弃   [e] 编辑   [s] 本次 session 不再提议
```

`e` 打开 `$EDITOR` 编辑后再确认。冲突时不直接覆盖，改问：

```text
与 m-91cc 冲突：
  旧：Rust 项目用 cargo test
  新：Rust 仓库一律用 cargo nextest
  [r] 替换   [k] 并存   [d] 丢弃
```

### 5.3 journal

自动写，不确认，但必须守规则：

- 3–5 行，只写**与上次不同的东西**：决定、转折、未解问题、情绪变化。
- 不写过程流水账（「今天修了个 bug」），不重复 `memory/` 已有内容。
- 只记事件和判断，不记代码细节；引用的路径可以留，代码不抄。
- 写入前过隐私扫描（transcript 里可能带 key）。

journal 不是日志，是共同经历的压缩。判断标准：三个月后读这一段，还能不能想起当时的取舍。

### 5.4 loops

- 它说「我下次帮你查」「回头补上」时，必须写 `loops.md`，否则信任会崩。
- 每条带可选的截止日期；没有截止的，两周后在启动时提示一次。
- 关闭 loop 需要证据（一次验证、一个结论），不能只是「觉得做完了」。

## 6. 过期、冲突、隐私

### 6.1 软过期

| 年龄（距 `最后确认`） | 注入行为 |
|---|---|
| ≤ 30 天 | 正常注入 |
| 31–90 天 | 注入，标注 `(unverified since YYYY-MM-DD)` |
| > 90 天 | 不自动注入，只在 `/mem recall` 时出现 |

不做自动删除。删除不可逆，而且你永远不会知道它删了什么。`/mem review` 可以批量重确认或清理。

`journal` 不适用软过期——经历不会失效，只是变旧。`loops` 用截止日期，不用年龄。

### 6.2 冲突

新旧记忆在写入前做一次廉价比对：

- 归一化（小写、去标点）后取 token 集合，Jaccard ≥ 0.5 视为可能冲突；
- 标签相同且文本相似度 ≥ 0.35 也视为可能冲突；
- 命中则弹「替换 / 并存 / 丢弃」，不自动合并。

阈值先定 0.5，跑两周后按误报调整。不上 embedding：200 条规模下，向量检索带来的复杂度远大于收益。

### 6.3 隐私

记忆每一轮都会发给模型 provider，所以泄露频率远高于普通文件。写入前扫描，命中直接拒绝并说明原因：

| 模式 | 例子 |
|---|---|
| `sk-[A-Za-z0-9]{20,}` | OpenAI / DeepSeek key |
| `ghp_` / `github_pat_` | GitHub token |
| `AKIA[0-9A-Z]{16}` | AWS access key |
| `-----BEGIN .* PRIVATE KEY-----` | 私钥 |
| `xox[baprs]-` | Slack token |
| 连续 32 位以上 hex / base64 | 通用密钥形态 |

扫描是兜底，不是保证。未公开项目、客户名、内部地址仍然靠确认时的人判断。`NIUSE_MEMORY=off` 可以整体关闭注入，用于 A/B 和信任受损时的降级。

## 7. 冷启动

`niuse bootstrap` 一次性把历史变成 `user.md` 和候选记忆：

1. 枚举 `~/.niucode/sessions/*.jsonl`，取最近 90 天、最多 40 个；优先多轮且有工具调用的会话。
2. 每个 session 只送 user / assistant 文本，**丢掉工具输出**，做一次无工具的抽取，产出 `{type, text, evidence}`。
3. 去重、按类型聚合，写入 `~/.niuse/inbox.md`，每条带来源 session id。
4. `niuse review` 批量过一遍：`y` 接受、`n` 丢弃、`e` 编辑。
5. `identity.md` 不挖掘：`niuse identity init` 问你八个问题，你口述，它落盘，你再改。

成本估算：40 个 session × 约 2k 输入 token ≈ 10 万 token 以内，一次性的。

挖掘的边界：只挖 D2 定义的三类，不挖代码事实、不挖环境配置、不挖别人的信息。

## 8. 架构

```text
crates/niuse-cli     REPL、命令、renderer、driver、bootstrap
    │
crates/niuse-core    date / model / store / prompt / candidate / policy
    │
niucode-agent        run_with_history / AgentConfig / project_history
niucode-core         SessionLog / Event
niucode-provider / auth / tools
```

- 两个 crate：`niuse-core` 只依赖 `anyhow` 与 `regex`，35 个单测不需要 API key；`niuse-cli` 负责终端、provider 装配与驱动。
- 依赖当前用 path 指向同级检出（`../niuverse/niucode`）。发布或换机器时改为 git + 固定 rev，不跟随 main。
- 中间一层 `NiucodeDriver`（目标 200 行以内）隔离 API 变动：config 组装、history 投影、事件转发。
- `niucode-cli` 只有 bin、没有 lib，MVP 不复用它的 TUI 和渲染，自己写一个最小 renderer。等上游拆出 lib 或加 hook 再换皮。
- 不改 niucode 的 loop，不给它加 memory 钩子。记忆是产品层，内核保持最小。

## 9. W1 必须验证的三个技术假设

写在纸上不算数，第一周用 spike 验证；不成立就改方案：

1. **同一 session 逐 turn 切换 cwd**：能，则是一条关系主线；不能，退化为按项目分会话，伙伴感打折，需要重新设计线程模型。
2. **流式输出可靠剥离尾部候选块**：需要一个约 120 字符的尾部缓冲，且不能破坏中止、续写与 diff 渲染。
3. **`niucode-agent` 的公共 API 稳定到一个薄适配层就够**：跑通一次真实任务，记录需要绕过的地方；超过 300 行说明耦合过深，回头和 niucode 商量加 hook。

### W1 结果（2026-10-21）

三条全部通过，实现按原方案推进：

1. **通过。** 同一条 `thread.jsonl` 里先在小目录读文件，`/cd /tmp` 后继续读另一个目录，两轮都成功；history 由 `project_history` 重建，未出现错位。
2. **通过。** `StreamFilter` 覆盖了标记跨 chunk、部分前缀、伪标记、未闭合块六种边界；真实调用中候选块被剥离，正文完整。
3. **通过。** `driver.rs` 约 200 行，没有需要 fork 或绕过 loop 的地方。唯一需要自己实现的是 renderer——`niucode-cli` 只有 bin。

## 10. 三周计划

| 周 | 交付 | 状态 |
|---|---|---|
| W1 | 三个 spike；crate 骨架；五件套读写；prompt 组装；REPL 跑通一个真实任务 | 完成 |
| W2 | 候选块 + 确认 + 冲突替换；journal；loops；软过期；密钥扫描 | 完成（审批预览仍是摘要，不是完整 diff） |
| W3 | `niuse bootstrap`；dogfood；验收数据 | bootstrap / review 完成；dogfood 待做 |

## 11. 验收

技术项已通过（2026-10-21）：五件套读写与 0600、候选协议、流式剥离、软过期、密钥扫描、`bootstrap → review → memory` 全链路、同 session 换 cwd。

剩下的是产品验收，必须靠真实使用：

- [ ] 连续 10 个工作日，每天 ≥ 1 次真实任务经由 niuse
- [ ] 候选接受率 ≥ 60%（接受 / 提议）
- [ ] ≥ 3 次它引用你过去的判断，且你确认引用正确
- [ ] ≥ 1 次它明确反对你、你被说服
- [ ] 记忆库 ≤ 50 条（超过说明门槛太低）
- [ ] `NIUSE_MEMORY=off` 的 A/B 至少 1 个 case 输出不同

最后一条是防安慰剂：如果关掉记忆行为完全一样，那这一层就没有存在价值。

## 12. 不做

常驻进程、Web UI、微信 / 飞书通道、定时推送、向量检索、session 全量索引、自动改 repo、多人协作、TUI（MVP 阶段）、通用 agent 框架。

## 13. 风险与对策

| 风险 | 表现 | 对策 |
|---|---|---|
| 确认疲劳（头号） | 开始盲按 `y`，然后关掉功能 | 每 turn ≤ 1 条；拒绝后同类不再提；接受率可观测；`NIUSE_MEMORY=off` |
| 人格表演化 | 出现「我想你了」式台词 | `identity.md` 写死禁则；伙伴感只由记忆、承诺、反对承担 |
| journal 流水账 | 每天「修了个 bug」 | 只写变化与判断；每月 review 一次，删掉无信息量的段落 |
| 依赖腐烂 | niucode 升 rev 后编译不过 | 钉 rev；薄适配层；必要时向上游提 lib 拆分 PR |
| 冷启动噪声 | 挖出一堆「用户喜欢简洁代码」 | 只挖三类；候选必须带来源；批量 review 而不是全收 |

## 14. 待定

- 它的名字：`niuse` 是项目名，不一定是你叫它的名字；`identity.md` 里单独定义。
- bootstrap 的采样量与成本，跑一次再定。
- 是否允许它主动提议修改 `identity.md`：倾向允许提议、必须你确认。
- 单线程之外的会话模型：`/new`、按项目分会话，等真实使用暴露需求再定。

## 15. 实现对照与偏差

已实现（`crates/niuse-core` + `crates/niuse-cli`）：

| 计划项 | 落点 |
|---|---|
| 五件套与 0600 | `store.rs`，原子写 + `chmod` |
| 注入与预算 | `prompt.rs`，稳定段在前 |
| 协议块与流式剥离 | `candidate.rs`，含 6 个边界单测 |
| 软过期 / 冲突 / 密钥扫描 | `policy.rs` + `prompt.rs` |
| REPL、确认、`/mem` 等 | `repl.rs`、`commands.rs` |
| 会话恢复与 per-turn cwd | `driver.rs` |
| 冷启动 | `bootstrap.rs`（bootstrap + review） |

与计划的三处偏差：

1. **协议块多了 `journal`**。原计划只有 memory / loop；为了让 journal 不额外调用模型，改成由主模型在回答末尾带一个 journal 块，niuse 自动落盘。
2. **`exec` 不写任何记忆**。它只把候选打到 stderr，避免脚本化调用悄悄改库。
3. **审批只显示摘要与 +/- 行数**。结构化 unified diff 预览留给后续；不影响安全边界。
