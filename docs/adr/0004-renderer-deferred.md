+++
id = "adr-0004"
title = "ADR-0004：Renderer migration waits for TUI dogfood"
kind = "adr"
status = "accepted"
order = 4
summary = "Keep the rustyline REPL and its renderer as the default; TUI is opt-in until ten real TUI dogfood days and parity are proven."
+++

## 背景

ADR-0032 的 TUI seam 已经落地：niucode 0.1.5 提供 prompt / event / turn-end / filter / command / commands / after_turn hooks；niuse 0.1.11 的 `tui` feature 通过这些 hooks 跑通了协议剥离、journal/loops、memory 确认、审批 allow/deny、只读命令和 `/export`。

但 niuse 的默认入口仍是 rustyline REPL；`render.rs` 仍在服务它。现在删掉 renderer 会让默认路径失去经过最多真实使用的界面，也会把产品 UI 一次性绑到上游 TUI 的演进节奏上。

## 决策

1. **默认仍是 REPL + `render.rs`。** TUI 是显式的 `tui` feature，发布为可选安装。
2. **不搬文件。** renderer 删除发生在真实使用证明 TUI 路径之后，而不是在代码看起来可复用之后。
3. **删除门槛**：
   - 十个真实工作日的 TUI dogfood，每天至少一个真实任务；
   - 审批 allow/deny、memory accept/edit/replace/keep、`NIUSE_MEMORY=off`、`/export` 都无回归；
   - 至少一次 TUI 会话中 candidate 被接受并在一周后仍被正确引用；
   - 默认安装仍能只装 REPL，TUI 依赖不进入默认二进制。
4. **期间允许的改动**：只修 TUI 行为与 seam，不把新的产品逻辑写进 niucode。

## 备选

| 方案 | 否决理由 |
|---|---|
| 立刻删除 `render.rs`，TUI 转为默认 | 默认路径的回归面最大，dogfood 数据不足 |
| 永久保留两套 renderer | 长期双维护；ADR-0032 的收益无法兑现 |
| 把 TUI 变成默认、REPL 变 feature | 改变已发布默认行为，用户没有要求 |

## 后果

- 短期保留两套前端，成本是已知的；
- TUI 的每次改进都可以独立验证，不赌默认路径；
- 删除 renderer 是一个有明确验收的可回退决定，而不是仓库整理动作。

## 验收

- [x] TUI 为 opt-in feature，默认二进制不链接 TUI。
- [x] TUI 的 hooks 和确认路径有 PTY/单元测试。
- [ ] 十个真实 TUI dogfood 日。
- [ ] 删除 renderer 的独立提交。
