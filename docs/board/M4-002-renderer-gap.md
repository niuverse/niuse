+++
id = "M4-002"
title = "Renderer migration gap analysis"
status = "done"
milestone = "M4"
module = "frontend"
size = "M"
priority = "P1"
updated = "2026-10-22"
completed = "2026-10-22"
depends_on = ["M4-001"]
docs = ["adr-0003", "renderer-migration"]
+++

## 范围

- 对比 niuse `render.rs`（361 行）与 niucode TUI（约 3 960 行）的职责。
- 确认已有 seam：`augment_system_prompt`、`on_event`、`filter_text`、`on_turn_end`。
- `filter_text` 已实现并在 PTY 下验证：协议块可在显示前剥离，持久化事件不变。
- 明确还缺的两个产品 seam：candidate 确认交互、embedder 命令注册。
- 给出分阶段迁移计划，结论是**不直接搬文件**。

## 验收

- [x] 职责清单与代码规模量化。
- [x] 阻塞 gap 列清楚（确认、命令、审批、usage、config）。
- [x] `filter_text` 作为第一步落地并有运行时验证。
- [x] 迁移门槛写成十天 dogfood，不提前删除 renderer。

## 备注

详见 [renderer-migration](renderer-migration.md)。去留已写成 [ADR-0004](adr/0004-renderer-deferred.md)：TUI 先 opt-in，十个真实狗粮日后再删 renderer。
