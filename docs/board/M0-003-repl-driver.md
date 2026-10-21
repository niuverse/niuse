+++
id = "M0-003"
title = "REPL 与 niucode driver"
status = "done"
milestone = "M0"
module = "repl"
size = "L"
priority = "P0"
updated = "2026-10-21"
completed = "2026-10-21"
depends_on = ["M0-002"]
docs = ["architecture"]
+++

## 范围

- 交互式 REPL：流式输出、斜杠命令、Ctrl-C 取消、Ctrl-D 退出。
- driver 打开或恢复 `thread.jsonl`，用 `project_history` 重建历史。
- 每个 turn 重新设置 working dir，验证同 session 换 cwd。
- 候选确认：`y` / `n` / `e` / `s`，冲突时 `r` / `k` / `d`。

## 验收

- [x] 真实 provider 跑通只读任务，session 落盘。
- [x] 同一 session 先读小目录、`/cd /tmp` 后读另一目录，两轮都成功。
- [x] 确认后写入 memory，`niuse mem list` 可见。
- [x] journal / loop 块自动落盘。
- [x] driver 约 200 行，没有绕过 loop。

## 记录

`niucode-cli` 只有 bin、没有 lib，TUI 与 renderer 无法复用；MVP 自己写最小 renderer，等上游拆 lib 或加 hook 再换皮。
