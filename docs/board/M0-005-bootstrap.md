+++
id = "M0-005"
title = "历史冷启动与 review 队列"
status = "done"
milestone = "M0"
module = "bootstrap"
size = "M"
priority = "P1"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-004"]
docs = ["plan"]
+++

## 范围

- `niuse bootstrap [--limit N] [--since-days N]` 扫描 `~/.niucode/sessions`。
- 每个 session 只送 user / assistant 文本，工具输出丢弃。
- 一次无工具抽取，输出 `{kind, text, tags}`，写入 `inbox.md`。
- `niuse review` 逐条 `y` / `n` / `s` / `q`，接受后写入 memory。

## 验收

- [x] 空 session 被跳过，只对含对话的 session 调模型。
- [x] 与现有 memory 及同批候选做相似度去重。
- [x] inbox 格式往返有单测。
- [x] 真实跑通：挖出一条沙箱相关教训，review 接受后入库。

## 记录

默认上限 40 个 session。成本约每个 2k 输入 token，一次性。
