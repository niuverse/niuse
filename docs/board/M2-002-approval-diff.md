+++
id = "M2-002"
title = "审批显示完整 unified diff"
status = "backlog"
milestone = "M2"
module = "repl"
size = "S"
priority = "P2"
updated = "2026-10-21"
depends_on = ["M0-003"]
docs = ["architecture"]
+++

## 范围

- 审批时展示 `ChangePreview` 的结构化 diff，而不是只有摘要与 +/- 行数。
- 复用 niucode 的 `Toolbox::preview` 结果，不在 niuse 里复制路径策略。
- 长 diff 分页或折叠，保持终端可读。

## 验收

- [ ] write / edit 审批能看到真实 hunk。
- [ ] 审批拒绝后事件与行为一致。
- [ ] 非 TTY 退化为逐行确认。

## 记录

安全边界不变，只是把已有的预览信息显示出来。
