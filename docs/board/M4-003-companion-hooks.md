+++
id = "M4-003"
title = "niuse CompanionHooks adapter"
status = "done"
milestone = "M4"
module = "frontend"
size = "M"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M4-002"]
docs = ["renderer-migration"]
+++

## 范围

- 新增 `tui` feature，默认 niuse 二进制不依赖 niucode TUI。
- `CompanionHooks` 实现 `TurnHooks`：
  - `augment_system_prompt` 注入 niuse 的 composed prompt；
  - `filter_text` 用 `StreamFilter` 剥离协议块，同时保留 raw 文本；
  - `after_turn` 自动写 journal/loops，把 memory 变成 accept/discard prompt；
  - `on_answer` 写 memory 或记录 discard。
- 两个单元测试覆盖 accept 与 discard，且 loop 自动写入。
- 已接 `niuse tui`（feature）并用 replay fixture 做 PTY 端到端：协议块被剥离、出现 accept/discard 问题、accept 后 memory 落盘。

## 验收

- [x] 默认构建不引入 TUI 依赖。
- [x] `--features tui` 测试与 clippy 通过。
- [x] 协议块剥离、journal/loop 自动写入、memory prompt/写入有测试。
- [x] 真正的 `niuse tui` 入口与 replay PTY 端到端测试。
